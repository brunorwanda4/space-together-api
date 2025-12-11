use chrono::Weekday;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::school_timetable::{
        DailySchoolSchedule, SchoolTimetable, SchoolTimetablePartial, TimeBlock,
    },
    errors::AppError,
    models::{id_model::IdType, mongo_model::IndexDef},
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct SchoolTimetableService {
    pub collection: Collection<SchoolTimetable>,
}

impl SchoolTimetableService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SchoolTimetable>("school_timetables"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            // Core timetable identifiers
            IndexDef::compound(
                vec![("school_id", 1), ("academic_year_id", 1)],
                true, // unique: one timetable per school per year
            ),
            IndexDef::single("school_id", false),
            IndexDef::single("academic_year_id", false),
            // Overrides
            IndexDef::single("overrides.r#type", false),
            IndexDef::single("overrides.applies_to", false), // multikey
            IndexDef::compound(
                vec![("overrides.r#type", 1), ("overrides.applies_to", 1)],
                false,
            ),
            // Events
            IndexDef::single("events.event_id", true), // unique
            IndexDef::compound(
                vec![("events.start_date", 1), ("events.end_date", 1)],
                false,
            ),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let _ = repo.ensure_indexes(&indexes).await?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 1. CREATE
    // -------------------------------------------------------------------------
    pub async fn create(&self, dto: SchoolTimetable) -> Result<SchoolTimetable, AppError> {
        self.ensure_indexes().await?;

        // Check uniqueness for (school_id + academic_year_id)
        if let Ok(_) = self
            .find_by_school_and_academic_year(&dto.school_id, &dto.academic_year_id)
            .await
        {
            return Err(AppError {
                message: format!(
                    "School timetable for academic year {} already exists",
                    dto.academic_year_id
                ),
            });
        }

        // Validate
        if let Err(e) = dto.validate() {
            return Err(AppError {
                message: format!("Validation error: {}", e),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let partial = dto.to_partial();
        let doc = bson::to_document(&partial).map_err(|e| AppError {
            message: format!("Serialization error: {}", e),
        })?;

        repo.create::<SchoolTimetable>(doc, None).await
    }

    // -------------------------------------------------------------------------
    // 2. READ
    // -------------------------------------------------------------------------
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_filter: Option<Document>,
    ) -> Result<SchoolTimetable, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut filter = Document::new();

        if let Some(id_value) = id {
            let obj = IdType::to_object_id(id_value)?;
            filter.insert("_id", obj);
        }

        if let Some(extra) = extra_filter {
            for (k, v) in extra.into_iter() {
                filter.insert(k, v);
            }
        }

        let item = repo.find_one::<SchoolTimetable>(filter, None).await?;

        match item {
            Some(t) => Ok(t),
            None => Err(AppError {
                message: "School timetable not found".to_string(),
            }),
        }
    }

    pub async fn find_by_school_and_academic_year(
        &self,
        school_id: &ObjectId,
        academic_year_id: &ObjectId,
    ) -> Result<SchoolTimetable, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let filter = doc! {
            "school_id": school_id,
            "academic_year_id": academic_year_id
        };

        let item = repo.find_one::<SchoolTimetable>(filter, None).await?;

        match item {
            Some(t) => Ok(t),
            None => Err(AppError {
                message: "School timetable not found for this academic year".into(),
            }),
        }
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<crate::domain::common_details::Paginated<SchoolTimetable>, AppError> {
        let base = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["academic_year"];

        let (data, total, total_pages, current_page) = base
            .get_all::<SchoolTimetable>(filter, &searchable, limit, skip, None)
            .await?;

        Ok(crate::domain::common_details::Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // -------------------------------------------------------------------------
    // 3. UPDATE
    // -------------------------------------------------------------------------
    pub async fn update_timetable(
        &self,
        id: &IdType,
        dto: &SchoolTimetablePartial,
    ) -> Result<SchoolTimetable, AppError> {
        let obj_id = IdType::to_object_id(id)?;

        // Conflict check logic stays the same...
        if let (Some(sid), Some(ayid)) = (dto.school_id, &dto.academic_year_id) {
            if let Ok(existing) = self.find_by_school_and_academic_year(&sid, ayid).await {
                if existing.id != Some(obj_id) {
                    return Err(AppError {
                        message: "Timetable for this academic year already exists".into(),
                    });
                }
            }
        }
        // -----------------------------------------
        // Continue with update
        // -----------------------------------------
        let base = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let dto_clone = dto.clone();
        let doc = bson::to_document(&dto_clone).map_err(|e| AppError {
            message: format!("Serialize update error: {}", e),
        })?;

        let update_doc = extract_valid_fields(doc);

        base.update_one_and_fetch::<SchoolTimetable>(id, update_doc)
            .await
    }

    // -------------------------------------------------------------------------
    // 4. DELETE
    // -------------------------------------------------------------------------
    pub async fn delete_timetable(&self, id: &IdType) -> Result<SchoolTimetable, AppError> {
        let base = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let existing = self.find_one(Some(&id), None).await?;
        base.delete_one(id).await?;
        Ok(existing)
    }

    // -------------------------------------------------------------------------
    // 5. OPTIONAL: AUTO-GENERATE DEFAULT SCHOOL WEEK
    // -------------------------------------------------------------------------
    pub async fn generate_default(
        &self,
        school_id: &IdType,
        academic_year: &IdType,
    ) -> Result<SchoolTimetable, AppError> {
        let school_obj = IdType::to_object_id(school_id)?;
        let year_obj = IdType::to_object_id(academic_year)?;

        // Example default Mon–Fri schedule
        let days = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ];

        let mut weekly = vec![];

        for d in days {
            weekly.push(DailySchoolSchedule {
                day: d,
                is_school_day: true,
                school_start_time: "08:30".into(),
                school_end_time: "17:00".into(),
                study_start_time: "09:00".into(),
                study_end_time: "17:00".into(),
                breaks: vec![
                    TimeBlock {
                        start_time: "10:20".into(),
                        end_time: "10:40".into(),
                        description: Some("Morning break".into()),
                        title: "Break".into(),
                    },
                    TimeBlock {
                        start_time: "15:20".into(),
                        end_time: "15:40".into(),
                        description: Some("Afternoon break".into()),
                        title: "Break".into(),
                    },
                ],
                lunch: Some(TimeBlock {
                    start_time: "13:00".into(),
                    end_time: "14:00".into(),
                    description: Some("Time for lunch".into()),
                    title: "Lunch".into(),
                }),
                activities: vec![],
                special_type: crate::domain::school_timetable::DaySpecialType::Normal,
            });
        }

        let timetable = SchoolTimetable {
            id: None,
            school_id: school_obj,
            academic_year_id: year_obj,
            default_weekly_schedule: weekly,
            overrides: Some(vec![]),
            events: Some(vec![]),
            created_at: None,
            updated_at: None,
        };

        self.create(timetable).await
    }
}
