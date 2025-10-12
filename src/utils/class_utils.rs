// src/utils/class_utils.rs
use crate::domain::class::Class;

/// Sanitize a single class by removing sensitive information if needed
pub fn sanitize_class(class: Class) -> Class {
    // Currently just returning the class as-is
    // Add any sanitization logic here if needed in the future
    class
}

/// Sanitize multiple classes
pub fn sanitize_classes(classes: Vec<Class>) -> Vec<Class> {
    classes.into_iter().map(sanitize_class).collect()
}
