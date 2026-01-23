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
        // Generate the original struct
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type
            ),*
        }

        // Generate the partial struct with all fields as Option<T>
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        $vis struct $partial_name {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field_name : Option<$field_type>
            ),*
        }

        // Implementation methods for the original struct
        impl $name {
            /// Merge a partial struct into this instance, updating only fields that are Some()
            #[allow(dead_code)]
            pub fn merge(&mut self, partial: $partial_name) {
                $(
                    if let Some(val) = partial.$field_name {
                        self.$field_name = val;
                    }
                )*
            }

            /// Convert this struct to a partial representation with all fields as Some()
            #[allow(dead_code)]
            pub fn to_partial(&self) -> $partial_name {
                $partial_name {
                    $(
                        $field_name: Some(self.$field_name.clone()),
                    )*
                }
            }

            /// Converts this struct to a BSON document using the original schema's serialization
            /// This preserves all custom serialize_with attributes (like ObjectId handling)
            #[allow(dead_code)]
            pub fn to_document(&self) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                mongodb::bson::to_document(self)
                    .map_err(|e| $crate::errors::AppError {
                        message: format!("Failed to serialize to BSON document: {}", e)
                    })
            }

            /// Creates a new document from a partial, using only the fields that are Some()
            /// Uses the original schema's serialization for each field
            #[allow(dead_code)]
            pub fn from_partial(partial: $partial_name) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                use mongodb::bson::to_bson;
                let mut document = mongodb::bson::Document::new();

                $(
                    if let Some(value) = partial.$field_name {
                        let field_value = to_bson(&value)
                            .map_err(|e| $crate::errors::AppError {
                                message: format!("Failed to serialize field '{}': {}", stringify!($field_name), e)
                            })?;
                        document.insert(stringify!($field_name), field_value);
                    }
                )*

                Ok(document)
            }
        }
    };
}
