use crate::domain::auth_user::AuthUserDto;
use crate::domain::user_role::UserRole;
use actix_web::{error, Error};

/// Check if user is admin
pub fn is_admin(user: &AuthUserDto) -> bool {
    matches!(user.role, Some(UserRole::ADMIN))
}

/// Check if user is owner of resource or admin
pub fn is_owner_or_admin(user: &AuthUserDto, target_user_id: &str) -> bool {
    is_admin(user) || user.id == target_user_id
}

/// Check permission and return proper Error with message
pub fn check_owner_or_admin(user: &AuthUserDto, target_user_id: &str) -> Result<(), Error> {
    if user.id.is_empty() {
        // Not logged in
        Err(error::ErrorUnauthorized(
            "You must be logged in to perform this action",
        ))
    } else if is_owner_or_admin(user, target_user_id) {
        // Allowed
        Ok(())
    } else {
        // Logged in but not owner/admin
        Err(error::ErrorForbidden(
            "You are not allowed to access or modify this resource",
        ))
    }
}

/// Check only admin access
pub fn check_admin(user: &AuthUserDto) -> Result<(), Error> {
    if is_admin(user) {
        Ok(())
    } else {
        Err(error::ErrorForbidden("Only admins can perform this action"))
    }
}

/// Check if user is admin or school staff
pub fn check_admin_or_staff(user: &AuthUserDto) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) || user.role == Some(UserRole::SCHOOLSTAFF) {
        Ok(())
    } else {
        Err("Insufficient permissions. Requires Admin or School staff role.".to_string())
    }
}

/// Check if user has access to school operations
pub fn check_school_access(user: &AuthUserDto, _school_id: &str) -> Result<(), String> {
    // Admin has full access
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // School staff can access schools they're associated with
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        // Here you might want to check if the staff member belongs to this school
        // For now, we'll allow all school staff to update any school
        // You can implement more granular permissions later
        return Ok(());
    }

    Err("Insufficient permissions to access school".to_string())
}

pub fn check_admin_staff_or_teacher(user: &AuthUserDto) -> Result<(), String> {
    if user.role == Some(UserRole::SCHOOLSTAFF)
        || user.role == Some(UserRole::ADMIN)
        || user.role == Some(UserRole::TEACHER)
    {
        Ok(())
    } else {
        Err("Access denied: Admin, Staff, or Teacher role required".to_string())
    }
}

/// Check if user has access to the class (admin, class teacher, or school staff)
pub fn check_class_access(user: &AuthUserDto, class_id: &str) -> Result<(), String> {
    // Admin has access to everything
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // Check if user is the class teacher for this class
    if user.role == Some(UserRole::TEACHER) {
        // Use unwrap_or_default() to handle Option<Vec<String>>
        if user
            .accessible_classes
            .as_ref()
            .unwrap_or(&Vec::new())
            .contains(&class_id.to_string())
        {
            return Ok(());
        }
    }

    // Check if user is staff member of the school that owns this class
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        // For school staff, we can check if they have access to the school that owns this class
        // This would typically require a database query to get the school_id from the class_id
        // For now, we'll implement a basic check using the schools field
        if let Some(schools) = &user.schools {
            // In a real implementation, you would query the class to get its school_id
            // and check if that school_id is in the user's schools list
            // For now, we'll return true if the user has any schools (placeholder)
            if !schools.is_empty() {
                return Ok(());
            }
        }
    }

    Err("Access denied: No permission to access this class".to_string())
}

/// Check if user is admin or class teacher
pub fn check_admin_or_class_teacher(user: &AuthUserDto, class_id: &str) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    if user.role == Some(UserRole::TEACHER)
        && user
            .accessible_classes
            .as_ref()
            .unwrap_or(&Vec::new())
            .contains(&class_id.to_string())
    {
        return Ok(());
    }

    Err("Access denied: Admin or Class Teacher role required".to_string())
}

// ========== SUBJECT ACCESS CONTROL ==========

/// Check if user has access to subject (Admin, Staff, or Subject Teacher)
pub fn check_subject_access(user: &AuthUserDto, subject_id: &str) -> Result<(), String> {
    // Admin has full access
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // School staff can access subjects
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        return Ok(());
    }

    // For teachers, check if they are the class teacher for this subject
    if user.role == Some(UserRole::TEACHER) && is_subject_teacher(&user.id, subject_id) {
        return Ok(());
    }

    Err("Access denied: insufficient permissions for subject".to_string())
}

/// Check if user is Admin or Subject Teacher
pub fn check_admin_or_subject_teacher(user: &AuthUserDto, subject_id: &str) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    if user.role == Some(UserRole::TEACHER) && is_subject_teacher(&user.id, subject_id) {
        return Ok(());
    }

    Err("Access denied: must be admin or subject teacher".to_string())
}

/// Helper function to check if user is the subject teacher
/// You'll need to implement this based on your database structure
fn is_subject_teacher(user_id: &str, subject_id: &str) -> bool {
    // This is a placeholder implementation
    // You would typically:
    // 1. Parse the subject_id to ObjectId
    // 2. Query the subjects collection to find the subject
    // 3. Check if the user_id matches the class_teacher_id of the subject
    let _ = user_id;
    let _ = subject_id;
    // For now, return false - implement based on your actual data model
    // You might want to use your database connection to query the subject
    // and check if the class_teacher_id matches the user_id

    // Example of how this might look (pseudo-code):
    /*
    use crate::repositories::subject_repo::SubjectRepo;
    use crate::utils::object_id::parse_object_id;
    use crate::models::id_model::IdType;

    let subject_repo = SubjectRepo::new(&db_connection);
    let subject_id_parsed = IdType::from_string(subject_id);

    match subject_repo.find_by_id(&subject_id_parsed).await {
        Ok(Some(subject)) => {
            if let Some(class_teacher_id) = subject.class_teacher_id {
                // Compare the ObjectId with user_id (you might need to convert user_id to ObjectId)
                // This depends on how your user IDs are stored
                class_teacher_id.to_hex() == user_id
            } else {
                false
            }
        }
        _ => false
    }
    */

    // For now, we'll return a simple check based on accessible_classes
    // This assumes that if a teacher has access to a class, they might be teaching subjects in that class
    // Adjust this based on your actual data model
    false
}

/// Check if user has access to student (Admin, Staff, Teacher, or Student themselves)
pub fn check_student_access(user: &AuthUserDto, student_id: &str) -> Result<(), String> {
    // Admin has full access
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    // School staff can access students
    if user.role == Some(UserRole::SCHOOLSTAFF) {
        return Ok(());
    }

    // Teachers can access students
    if user.role == Some(UserRole::TEACHER) {
        return Ok(());
    }

    // Students can access their own data
    if user.role == Some(UserRole::STUDENT) {
        // Check if this student is accessing their own data
        // This assumes the student_id in the URL matches the user's student record ID
        // You might need to adjust this logic based on your data model
        if user.id == student_id {
            return Ok(());
        }
    }

    Err("Access denied: insufficient permissions for student".to_string())
}

/// Check if user is Admin or Student Creator
pub fn check_admin_or_student_creator(user: &AuthUserDto, student_id: &str) -> Result<(), String> {
    if user.role == Some(UserRole::ADMIN) {
        return Ok(());
    }

    let _ = student_id;

    // For non-admin users, we need to check if they created this student
    // This would typically require a database query to check the creator_id
    // For now, we'll implement a basic version that checks if the user is staff/teacher
    // You can enhance this with actual database checks later
    if user.role == Some(UserRole::SCHOOLSTAFF) || user.role == Some(UserRole::TEACHER) {
        // In a real implementation, you would:
        // 1. Parse the student_id to ObjectId
        // 2. Query the students collection to find the student
        // 3. Check if the user_id matches the creator_id of the student
        // For now, we'll return Ok() as a placeholder
        return Ok(());
    }

    Err("Access denied: must be admin or student creator".to_string())
}

// Helper function to check if user is the student creator
// You'll need to implement this based on your database structure
// fn is_student_creator(user_id: &str, student_id: &str) -> bool {
//     // This is a placeholder implementation
//     // You would typically:
//     // 1. Parse the student_id to ObjectId
//     // 2. Query the students collection to find the student
//     // 3. Check if the user_id matches the creator_id of the student

//     let _ = user_id;
//     let _ = student_id;

//     // Example of how this might look (pseudo-code):
//     /*
//     use crate::repositories::student_repo::StudentRepo;
//     use crate::utils::object_id::parse_object_id;
//     use crate::models::id_model::IdType;

//     let student_repo = StudentRepo::new(&db_connection);
//     let student_id_parsed = IdType::from_string(student_id);

//     match student_repo.find_by_id(&student_id_parsed).await {
//         Ok(Some(student)) => {
//             if let Some(creator_id) = student.creator_id {
//                 // Compare the ObjectId with user_id (you might need to convert user_id to ObjectId)
//                 creator_id.to_hex() == user_id
//             } else {
//                 false
//             }
//         }
//         _ => false
//     }
//     */
//     // For now, return true as a placeholder - implement based on your actual data model
//     true
// }

// Check if user is Admin, Staff, Teacher, or the Student themselves
// pub fn check_student_access_extended(user: &AuthUserDto, student_id: &str) -> Result<(), String> {
//     // Admin has full access
//     if user.role == Some(UserRole::ADMIN) {
//         return Ok(());
//     }

//     // School staff can access students
//     if user.role == Some(UserRole::SCHOOLSTAFF) {
//         return Ok(());
//     }

//     // Teachers can access students in their classes
//     if user.role == Some(UserRole::TEACHER) {
//         // You might want to add additional checks here to verify the teacher
//         // has this student in one of their classes
//         return Ok(());
//     }

//     // Students can only access their own data
//     if user.role == Some(UserRole::STUDENT) {
//         if is_student_user(user, student_id) {
//             return Ok(());
//         }
//         return Err("Students can only access their own data".to_string());
//     }

//     Err("Access denied: insufficient permissions for student access".to_string())
// }

// Helper function to check if user is the student
// fn is_student_user(user: &AuthUserDto, student_id: &str) -> bool {
//     // This function checks if the authenticated user is the same as the student
//     // being accessed. This depends on how your student-user relationship is structured.

//     // If your student records have a user_id field that matches the user's ID:
//     // You would typically query the student record and check if user_id matches

//     // For now, we'll do a simple string comparison as a placeholder
//     // You should replace this with actual database logic
//     user.id == student_id
// }

// Check if user can manage students (Admin, Staff, or Teacher)
// pub fn check_student_management(user: &AuthUserDto) -> Result<(), String> {
//     if user.role == Some(UserRole::ADMIN)
//         || user.role == Some(UserRole::SCHOOLSTAFF)
//         || user.role == Some(UserRole::TEACHER)
//     {
//         Ok(())
//     } else {
//         Err(
//             "Access denied: Admin, Staff, or Teacher role required for student management"
//                 .to_string(),
//         )
//     }
// }
