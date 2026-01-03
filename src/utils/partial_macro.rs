// ============================================
// THE MACROS
// ============================================

#[macro_export]
macro_rules! strip_option {
    (Option<$inner:ty>) => {
        $inner
    };
    ($other:ty) => {
        $other
    };
}

#[macro_export]
macro_rules! partial_type {
    // If it's Option<T>, keep it as Option<T>
    (Option<$inner:ty>) => { Option<$inner> };
    // If it's not Option, wrap it in Option
    ($other:ty) => { Option<$other> };
}

#[macro_export]
macro_rules! merge_field {
    // For Option<T> fields: replace if partial is Some
    ($self_field:expr, $partial_field:expr, Option<$inner:ty>) => {
        if $partial_field.is_some() {
            $self_field = $partial_field;
        }
    };
    // For non-Option fields: unwrap and assign if Some
    ($self_field:expr, $partial_field:expr, $field_type:ty) => {
        if let Some(val) = $partial_field {
            $self_field = val;
        }
    };
}

#[macro_export]
macro_rules! to_partial_field {
    // For Option<T> fields: just clone (stays Option<T>)
    ($self_field:expr, Option<$inner:ty>) => {
        $self_field.clone()
    };
    // For non-Option fields: wrap in Some
    ($self_field:expr, $field_type:ty) => {
        Some($self_field.clone())
    };
}

#[macro_export]
macro_rules! make_partial {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        } => $partial_name:ident
    ) => {
        // -------- Original Struct --------
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type
            ),*
        }

        // -------- Partial Struct --------
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        $vis struct $partial_name {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(default)]
                pub $field_name : $crate::partial_type!($field_type)
            ),*
        }

        // -------- Implementation --------
        impl $name {
            /// Merge partial values into this struct (only updates fields that are Some)
            #[allow(dead_code)]
            pub fn merge(&mut self, partial: $partial_name) {
                $(
                    $crate::merge_field!(self.$field_name, partial.$field_name, $field_type);
                )*
            }

            /// Convert this struct to a partial (all fields become Some)
            #[allow(dead_code)]
            pub fn to_partial(&self) -> $partial_name {
                $partial_name {
                    $(
                        $field_name: $crate::to_partial_field!(&self.$field_name, $field_type),
                    )*
                }
            }
        }
    };
}
