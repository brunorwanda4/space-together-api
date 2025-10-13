use crate::domain::subject::Subject;

/// Sanitize a single subject by clearing the ID (for responses)
pub fn sanitize_subject(subject: Subject) -> Subject {
    // subject.id = None;
    subject
}

/// Sanitize multiple subjects by clearing their IDs (for responses)
pub fn sanitize_subjects(subjects: Vec<Subject>) -> Vec<Subject> {
    subjects.into_iter().map(sanitize_subject).collect()
}

// /// Validate subject category
// pub fn is_valid_subject_category(category: &str) -> bool {
//     !category.trim().is_empty()
// }

// /// Extract subject tags for search
// pub fn extract_search_tags(subject: &Subject) -> Vec<String> {
//     let mut tags = subject.tags.clone();
//     tags.push(subject.name.clone());
//     tags.push(subject.subject_type.to_string());

//     if let Some(ref description) = subject.description {
//         tags.push(description.clone());
//     }

//     tags
// }

// /// Format subject display name
// pub fn format_subject_display_name(subject: &Subject) -> String {
//     if let Some(ref code) = subject.code {
//         format!("{} ({})", subject.name, code)
//     } else {
//         subject.name.clone()
//     }
// }
