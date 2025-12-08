use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        class_timetable::{ClassTimetable, ClassTimetablePartial, WeekSchedule},
        common_details::Paginated,
    },
    errors::AppError,
    models::id_model::IdType,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct ClassTimetableService {
    pub collection: Collection<ClassTimetable>,
}

impl ClassTimetableService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<ClassTimetable>("class_timetables"),
        }
    }

    // -------------------------------------------------------------------------
    // 1. Create (Standard)
    // -------------------------------------------------------------------------
    pub async fn create(&self, dto: ClassTimetable) -> Result<ClassTimetable, AppError> {
        // 1. Check if a timetable already exists for this class and year
        if let Ok(_) = self
            .find_by_class_and_year(&dto.class_id, &dto.academic_year)
            .await
        {
            return Err(AppError {
                message: format!(
                    "Timetable for class {} in year {} already exists",
                    dto.class_id, dto.academic_year
                ),
            });
        }

        // 2. Validate the schedule structure (overlaps, start times)
        for week_schedule in &dto.weekly_schedule {
            if let Err(e) = week_schedule.validate() {
                return Err(AppError {
                    message: format!("Validation error on {}: {}", week_schedule.day, e),
                });
            }
        }

        // 3. Prepare document
        let new_timetable = dto.to_partial();

        let full_doc = bson::to_document(&new_timetable).map_err(|e| AppError {
            message: format!("Failed to serialize create: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<ClassTimetable>(full_doc, None).await
    }

    // -------------------------------------------------------------------------
    // 2. Read (Find One, Find All, Find by Class)
    // -------------------------------------------------------------------------
    pub async fn find_one_by_id(&self, id: &IdType) -> Result<ClassTimetable, AppError> {
        let obj = IdType::to_object_id(id)?;
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! { "_id": obj };
        let item = repo.find_one::<ClassTimetable>(filter, None).await?;

        match item {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Class timetable not found".to_string(),
            }),
        }
    }

    pub async fn find_by_class_and_year(
        &self,
        class_id: &ObjectId,
        academic_year: &str,
    ) -> Result<ClassTimetable, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {
            "class_id": class_id,
            "academic_year": academic_year
        };

        let item = repo.find_one::<ClassTimetable>(filter, None).await?;

        match item {
            Some(s) => Ok(s),
            None => Err(AppError {
                message: "Class timetable not found for this class/year".to_string(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Paginated<ClassTimetable>, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Fields to search against string filter
        let searchable = ["academic_year"];

        let (data, total, total_pages, current_page) = base_repo
            .get_all::<ClassTimetable>(filter, &searchable, limit, skip, None)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // -------------------------------------------------------------------------
    // 3. Update
    // -------------------------------------------------------------------------
    pub async fn update_timetable(
        &self,
        id: &IdType,
        update: &ClassTimetablePartial,
    ) -> Result<ClassTimetable, AppError> {
        let obj_id = IdType::to_object_id(id)?;

        // Ensure we aren't updating to a class/year combo that already exists elsewhere
        if let (Some(cid), Some(year)) = (update.class_id, &update.academic_year) {
            if let Ok(existing) = self.find_by_class_and_year(&cid, year).await {
                if existing.id != Some(obj_id) {
                    return Err(AppError {
                        message: "Timetable for this class/year already exists".into(),
                    });
                }
            }
        }

        // Validate nested periods if they are being updated
        if let Some(schedule) = &update.weekly_schedule {
            for day_sched in schedule {
                if let Err(e) = day_sched.validate() {
                    return Err(AppError {
                        message: format!("Invalid schedule update: {}", e),
                    });
                }
            }
        }

        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        // Add updated_at
        let update_clone = update.clone();

        let full_doc = bson::to_document(&update_clone).map_err(|e| AppError {
            message: format!("Failed to serialize update: {}", e),
        })?;

        let update_doc = extract_valid_fields(full_doc);

        base_repo
            .update_one_and_fetch::<ClassTimetable>(id, update_doc)
            .await
    }

    // -------------------------------------------------------------------------
    // 4. Delete
    // -------------------------------------------------------------------------
    pub async fn delete_timetable(&self, id: &IdType) -> Result<ClassTimetable, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let item = self.find_one_by_id(id).await?; // Fetch first to return it
        base_repo.delete_one(id).await?;
        Ok(item)
    }

    // -------------------------------------------------------------------------
    // 5. Specialized: Generate Empty Structure
    // -------------------------------------------------------------------------
    /// Helper to initialize a default Mon-Fri structure that the frontend can then fill
    pub fn generate_default_structure(
        class_id: ObjectId,
        year: String,
        start_time: &str,
    ) -> ClassTimetable {
        use chrono::Weekday::*;

        let days = vec![Mon, Tue, Wed, Thu, Fri];
        let mut weekly_schedule = Vec::new();

        for day in days {
            weekly_schedule.push(WeekSchedule {
                day,
                is_holiday: false,
                start_on: Some(start_time.to_string()),
                periods: vec![], // Empty initially
            });
        }

        ClassTimetable {
            id: None,
            class_id,
            academic_year: year,
            weekly_schedule,
            created_at: None,
            updated_at: None,
            disabled: None,
        }
    }
}
