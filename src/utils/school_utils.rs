use crate::domain::school::School;

/// Sanitize a single school by removing sensitive fields
pub fn sanitize_school(school: School) -> School {
    // TODO: to remover school data which are unique example code, database name
    // Remove any sensitive fields if needed
    // For schools, you might want to remove internal fields or format data
    // Currently no sensitive fields to remove, but keeping the structure for consistency

    // You can add any school-specific sanitization logic here
    // For example: format phone numbers, normalize addresses, etc.
    school
}

/// Sanitize multiple schools
pub fn sanitize_schools(schools: Vec<School>) -> Vec<School> {
    schools.into_iter().map(sanitize_school).collect()
}
