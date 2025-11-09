use crate::{
    config::state::AppState,
    domain::{
        common_details::Gender,
        student::{
            BulkStudentIds, BulkStudentTags, BulkUpdateStudentStatus, Student, StudentStatus,
            StudentWithRelations, UpdateStudent,
        },
    },
    helpers::object_id_helpers::parse_object_id,
    models::id_model::IdType,
    repositories::student_repo::StudentRepo,
    services::{cloudinary_service::CloudinaryService, event_service::EventService},
    utils::{email::is_valid_email, names::is_valid_name},
};
use actix_web::web;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct StudentService<'a> {
    repo: &'a StudentRepo,
}

impl<'a> StudentService<'a> {
    pub fn new(repo: &'a StudentRepo) -> Self {
        Self { repo }
    }

    // ------------------------------------------------------------------
    // ✅ CRUD OPERATIONS
    // ------------------------------------------------------------------

    /// Get all students
    pub async fn get_all_students(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .get_all_students(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Get all students with relations
    pub async fn get_all_students_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<StudentWithRelations>, String> {
        self.repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)
    }

    /// Get active students
    pub async fn get_active_students(&self) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .get_active_students()
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Create a new student
    pub async fn create_student(&self, mut new_student: Student) -> Result<Student, String> {
        // Validate name
        if let Err(e) = is_valid_name(&new_student.name) {
            return Err(format!("Invalid student name: {}", e));
        }

        // Validate email
        if let Err(e) = is_valid_email(&new_student.email) {
            return Err(format!("Invalid email: {}", e));
        }

        // Check if email already exists
        if let Ok(Some(_)) = self.repo.find_by_email(&new_student.email).await {
            return Err("Student email already exists".to_string());
        }

        // Check if user_id already exists (if provided)
        if let Some(user_id) = &new_student.user_id {
            if let Ok(Some(_)) = self
                .repo
                .find_by_user_id(&IdType::from_object_id(*user_id))
                .await
            {
                return Err("User ID already associated with another student".to_string());
            }
        }

        // Check if registration number already exists (if provided)
        if let Some(reg_number) = &new_student.registration_number {
            if let Ok(Some(_)) = self.repo.find_by_registration_number(reg_number).await {
                return Err("Registration number already exists".to_string());
            }
        }

        if let Some(image_data) = new_student.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data).await?;
            new_student.image_id = Some(cloud_res.public_id);
            new_student.image = Some(cloud_res.secure_url);
        }

        // Set timestamps
        let now = Utc::now();
        new_student.created_at = now;
        new_student.updated_at = now;

        // Set default values for optional fields
        if !new_student.is_active {
            new_student.is_active = true;
        }

        // Ensure tags is initialized
        if new_student.tags.is_empty() {
            new_student.tags = Vec::new();
        }

        // Set default status if not provided
        if matches!(new_student.status, StudentStatus::Active) {
            new_student.status = StudentStatus::Active;
        }

        // Generate ID
        let student_id = ObjectId::new();
        new_student.id = Some(student_id);

        // Save student in database
        let inserted_student = self
            .repo
            .insert_student(&new_student)
            .await
            .map_err(|e| e.message)?;

        Ok(inserted_student)
    }

    /// Get student by ID
    pub async fn get_student_by_id(&self, id: &IdType) -> Result<Student, String> {
        let student = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        Ok(student)
    }

    /// Get student by ID with relations
    pub async fn get_student_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<StudentWithRelations, String> {
        let student = self
            .repo
            .find_by_id_with_relations(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        Ok(student)
    }

    /// Get student by user ID
    pub async fn get_student_by_user_id(&self, user_id: &IdType) -> Result<Student, String> {
        let student = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found for this user".to_string())?;

        Ok(student)
    }

    /// Get student by email
    pub async fn get_student_by_email(&self, email: &str) -> Result<Student, String> {
        let student = self
            .repo
            .find_by_email(email)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        Ok(student)
    }

    /// Get student by registration number
    pub async fn get_student_by_registration_number(
        &self,
        registration_number: &str,
    ) -> Result<Student, String> {
        let student = self
            .repo
            .find_by_registration_number(registration_number)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        Ok(student)
    }

    /// Get students by school ID
    pub async fn get_students_by_school_id(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .find_by_school_id(school_id)
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Get students by class ID
    pub async fn get_students_by_class_id(
        &self,
        class_id: &IdType,
    ) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .find_by_class_id(class_id)
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Get students by creator ID
    pub async fn get_students_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .find_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Get students by status
    pub async fn get_students_by_status(
        &self,
        status: StudentStatus,
    ) -> Result<Vec<Student>, String> {
        let students = self
            .repo
            .find_by_status(status)
            .await
            .map_err(|e| e.message)?;
        Ok(students)
    }

    /// Update a student
    pub async fn update_student(
        &self,
        id: &IdType,
        mut updated_data: UpdateStudent,
    ) -> Result<Student, String> {
        // Validate name if provided
        if let Some(ref name) = updated_data.name {
            if let Err(e) = is_valid_name(name) {
                return Err(format!("Invalid student name: {}", e));
            }
        }

        // Validate email if provided
        if let Some(ref email) = updated_data.email {
            if let Err(e) = is_valid_email(email) {
                return Err(format!("Invalid email: {}", e));
            }
        }

        let existing_student = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        // Check email uniqueness if provided and changed
        if let Some(ref email) = updated_data.email {
            if existing_student.email != *email {
                if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                    return Err("Student email already exists".to_string());
                }
            }
        }

        // Check registration number uniqueness if provided and changed
        if let Some(ref reg_number) = updated_data.registration_number {
            if existing_student.registration_number.as_ref() != Some(reg_number) {
                if let Ok(Some(_)) = self.repo.find_by_registration_number(reg_number).await {
                    return Err("Registration number already exists".to_string());
                }
            }
        }

        // ☁️ Replace profile image
        if let Some(new_image_data) = updated_data.image.clone() {
            if Some(new_image_data.clone()) != existing_student.image {
                if let Some(old_image_id) = existing_student.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data).await?;
                updated_data.image_id = Some(cloud_res.public_id);
                updated_data.image = Some(cloud_res.secure_url);
            }
        }

        // Update student using repository method
        let updated_student = self
            .repo
            .update_student(id, &updated_data)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_student)
    }

    /// Delete a student by id
    pub async fn delete_student(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_student(id).await.map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ DELETE WITH EVENTS
    // ------------------------------------------------------------------

    pub async fn delete_student_with_events(
        &self,
        id: &IdType,
        state: &web::Data<AppState>,
    ) -> Result<(), String> {
        // Get student before deletion for broadcasting
        let student = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Student not found".to_string())?;

        self.delete_student(id).await?;

        // 🔔 Broadcast student deletion event
        if let Some(student_id) = &student.id {
            Self::broadcast_student_deletion(state, student_id, &student).await;
        }

        Ok(())
    }

    /// Count students by school ID
    pub async fn count_students_by_school_id(
        &self,
        school_id: &IdType,
        gender: Option<Gender>,
        status: Option<StudentStatus>,
    ) -> Result<u64, String> {
        self.repo
            .count_by_school_id(school_id, gender, status)
            .await
            .map_err(|e| e.message)
    }

    /// Count students by class ID
    pub async fn count_students_by_class_id(&self, class_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_class_id(class_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count students by creator ID
    pub async fn count_students_by_creator_id(&self, creator_id: &IdType) -> Result<u64, String> {
        self.repo
            .count_by_creator_id(creator_id)
            .await
            .map_err(|e| e.message)
    }

    /// Count students by status
    pub async fn count_students_by_status(&self, status: StudentStatus) -> Result<u64, String> {
        self.repo
            .count_by_status(status)
            .await
            .map_err(|e| e.message)
    }

    // ------------------------------------------------------------------
    // ✅ BULK OPERATIONS WITH EVENTS
    // ------------------------------------------------------------------

    /// Create multiple students
    pub async fn create_many_students(
        &self,
        students: Vec<Student>,
    ) -> Result<Vec<Student>, String> {
        // Validate all students first
        for student in &students {
            if let Err(e) = is_valid_name(&student.name) {
                return Err(format!("Invalid student name '{}': {}", student.name, e));
            }

            if let Err(e) = is_valid_email(&student.email) {
                return Err(format!("Invalid email '{}': {}", student.email, e));
            }
        }

        // Process students: set timestamps, etc.
        let mut processed_students = Vec::with_capacity(students.len());
        let now = Utc::now();

        for mut student in students {
            // Set timestamps
            student.created_at = now;
            student.updated_at = now;

            // Set default values for optional fields
            if !student.is_active {
                student.is_active = true;
            }

            // Ensure tags is initialized
            if student.tags.is_empty() {
                student.tags = Vec::new();
            }

            // Set default status if not provided
            if matches!(student.status, StudentStatus::Active) {
                student.status = StudentStatus::Active;
            }

            // Generate ID
            student.id = Some(ObjectId::new());

            processed_students.push(student);
        }

        // Create students using repository
        let created_students = self
            .repo
            .create_many_students(processed_students)
            .await
            .map_err(|e| e.message)?;

        Ok(created_students)
    }

    /// Create multiple students with events
    pub async fn create_many_students_with_events(
        &self,
        students: Vec<Student>,
        state: &web::Data<AppState>,
    ) -> Result<Vec<Student>, String> {
        let created_students = self.create_many_students(students).await?;

        // 🔔 Broadcast creation events for all created students
        for student in &created_students {
            if let Some(id) = &student.id {
                Self::broadcast_student_update(state, id).await;
            }
        }

        Ok(created_students)
    }

    /// Bulk update multiple students
    pub async fn update_many_students(
        &self,
        updates: Vec<(IdType, UpdateStudent)>,
    ) -> Result<Vec<Student>, String> {
        // Validate all updates first
        for (_, update) in &updates {
            if let Some(ref name) = update.name {
                if let Err(e) = is_valid_name(name) {
                    return Err(format!("Invalid student name '{}': {}", name, e));
                }
            }

            if let Some(ref email) = update.email {
                if let Err(e) = is_valid_email(email) {
                    return Err(format!("Invalid email '{}': {}", email, e));
                }
            }
        }

        // Check uniqueness for emails that are being changed
        for (id, update) in &updates {
            if let Some(ref email) = update.email {
                // Get existing student to check if email is changing
                if let Ok(Some(existing_student)) = self.repo.find_by_id(id).await {
                    if existing_student.email != *email {
                        if let Ok(Some(_)) = self.repo.find_by_email(email).await {
                            return Err(format!("Student email already exists: {}", email));
                        }
                    }
                }
            }

            if let Some(ref reg_number) = update.registration_number {
                // Get existing student to check if registration number is changing
                if let Ok(Some(existing_student)) = self.repo.find_by_id(id).await {
                    if existing_student.registration_number.as_ref() != Some(reg_number) {
                        if let Ok(Some(_)) = self.repo.find_by_registration_number(reg_number).await
                        {
                            return Err(format!(
                                "Registration number already exists: {}",
                                reg_number
                            ));
                        }
                    }
                }
            }
        }

        // Perform bulk update
        let updated_students = self
            .repo
            .update_many_students(updates)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_students)
    }

    /// Bulk update status for multiple students
    pub async fn bulk_update_status(
        &self,
        request: &BulkUpdateStudentStatus,
    ) -> Result<Vec<Student>, String> {
        let updated_students = self
            .repo
            .bulk_update_status(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_students)
    }

    /// Bulk add tags to multiple students
    pub async fn bulk_add_tags(&self, request: &BulkStudentTags) -> Result<Vec<Student>, String> {
        let updated_students = self
            .repo
            .bulk_add_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_students)
    }

    /// Bulk remove tags from multiple students
    pub async fn bulk_remove_tags(
        &self,
        request: &BulkStudentTags,
    ) -> Result<Vec<Student>, String> {
        let updated_students = self
            .repo
            .bulk_remove_tags(request)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_students)
    }

    /// Delete multiple students
    pub async fn delete_many_students(&self, request: &BulkStudentIds) -> Result<u64, String> {
        self.repo
            .delete_many_students(request)
            .await
            .map_err(|e| e.message)
    }

    /// Transfer students to another class
    pub async fn transfer_students_to_class(
        &self,
        student_ids: &BulkStudentIds,
        new_class_id: &IdType,
    ) -> Result<Vec<Student>, String> {
        let updated_students = self
            .repo
            .transfer_students_to_class(student_ids, new_class_id)
            .await
            .map_err(|e| e.message)?;

        Ok(updated_students)
    }

    // ------------------------------------------------------------------
    // 🔔 EVENT BROADCASTING METHODS
    // ------------------------------------------------------------------

    /// Broadcast student update event
    async fn broadcast_student_update(state: &web::Data<AppState>, student_id: &ObjectId) {
        let state_clone = state.clone();
        let student_id_clone = *student_id;

        actix_rt::spawn(async move {
            // Fetch the updated student with relations for broadcasting
            let repo = StudentRepo::new(&state_clone.db.main_db());
            if let Ok(Some(updated_student)) = repo
                .find_by_id_with_relations(&IdType::from_object_id(student_id_clone))
                .await
            {
                EventService::broadcast_updated(
                    &state_clone,
                    "student",
                    &student_id_clone.to_hex(),
                    &updated_student,
                )
                .await;
            }
        });
    }

    /// Broadcast student deletion event
    async fn broadcast_student_deletion(
        state: &web::Data<AppState>,
        student_id: &ObjectId,
        student: &Student,
    ) {
        let state_clone = state.clone();
        let student_id_clone = *student_id;
        let student_clone = student.clone();

        actix_rt::spawn(async move {
            EventService::broadcast_deleted(
                &state_clone,
                "student",
                &student_id_clone.to_hex(),
                &student_clone,
            )
            .await;
        });
    }

    // ------------------------------------------------------------------
    // 🔧 UTILITY METHODS
    // ------------------------------------------------------------------

    /// Get students by admission year
    pub async fn get_students_by_admission_year(
        &self,
        admission_year: i32,
        school_id: Option<&IdType>,
    ) -> Result<Vec<Student>, String> {
        let all_students = self.get_all_students(None, None, None).await?;

        let filtered_students: Vec<Student> = all_students
            .into_iter()
            .filter(|student| {
                if let Some(year) = student.admission_year {
                    if year != admission_year {
                        return false;
                    }
                } else {
                    return false;
                }

                // Filter by school_id if provided
                if let Some(school_id) = school_id {
                    if let Some(student_school_id) = student.school_id {
                        let target_school_id = match parse_object_id(school_id) {
                            Ok(id) => id,
                            Err(_) => return false,
                        };
                        return student_school_id == target_school_id;
                    } else {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered_students)
    }

    /// Check if a user is a student of a specific school
    pub async fn is_user_student_of_school(
        &self,
        user_id: &IdType,
        school_id: &IdType,
    ) -> Result<bool, String> {
        let student = self
            .repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| e.message)?;

        if let Some(student) = student {
            // Compare school IDs
            if let Some(student_school_id) = student.school_id {
                let target_school_id = parse_object_id(school_id)?;
                return Ok(student_school_id == target_school_id);
            }
        }

        Ok(false)
    }

    /// Check if a student is in a specific class
    pub async fn is_student_in_class(
        &self,
        student_id: &IdType,
        class_id: &IdType,
    ) -> Result<bool, String> {
        let student = self
            .repo
            .find_by_id(student_id)
            .await
            .map_err(|e| e.message)?;

        if let Some(student) = student {
            if let Some(student_class_id) = student.class_id {
                let target_class_id = parse_object_id(class_id)?;
                return Ok(student_class_id == target_class_id);
            }
        }

        Ok(false)
    }

    /// Get students with specific status in a school
    pub async fn get_school_students_by_status(
        &self,
        school_id: &IdType,
        status: StudentStatus,
    ) -> Result<Vec<Student>, String> {
        let school_students = self.get_students_by_school_id(school_id).await?;

        let filtered_students: Vec<Student> = school_students
            .into_iter()
            .filter(|student| student.status == status)
            .collect();

        Ok(filtered_students)
    }

    /// Get class roster with student details
    pub async fn get_class_roster(
        &self,
        class_id: &IdType,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<StudentWithRelations>, String> {
        let students = self
            .repo
            .get_all_with_relations(filter, limit, skip)
            .await
            .map_err(|e| e.message)?;

        let class_obj_id = parse_object_id(class_id)?;
        let roster: Vec<StudentWithRelations> = students
            .into_iter()
            .filter(|student_with_rel| {
                student_with_rel
                    .student
                    .class_id
                    .map(|id| id == class_obj_id)
                    .unwrap_or(false)
            })
            .collect();

        Ok(roster)
    }

    /// Update student status with validation
    pub async fn update_student_status(
        &self,
        student_id: &IdType,
        new_status: StudentStatus,
    ) -> Result<Student, String> {
        let update_data = UpdateStudent {
            status: Some(new_status),
            ..Default::default()
        };

        self.update_student(student_id, update_data).await
    }

    /// Suspend a student
    pub async fn suspend_student(&self, student_id: &IdType) -> Result<Student, String> {
        self.update_student_status(student_id, StudentStatus::Suspended)
            .await
    }

    /// Activate a student
    pub async fn activate_student(&self, student_id: &IdType) -> Result<Student, String> {
        let update_data = UpdateStudent {
            status: Some(StudentStatus::Active),
            is_active: Some(true),
            ..Default::default()
        };

        self.update_student(student_id, update_data).await
    }

    /// Graduate a student
    pub async fn graduate_student(&self, student_id: &IdType) -> Result<Student, String> {
        self.update_student_status(student_id, StudentStatus::Graduated)
            .await
    }

    /// Get student statistics for a school
    pub async fn get_school_student_statistics(
        &self,
        school_id: &IdType,
    ) -> Result<std::collections::HashMap<StudentStatus, u64>, String> {
        let students = self.get_students_by_school_id(school_id).await?;

        let mut stats = std::collections::HashMap::new();
        for student in students {
            *stats.entry(student.status).or_insert(0) += 1;
        }

        Ok(stats)
    }
}
