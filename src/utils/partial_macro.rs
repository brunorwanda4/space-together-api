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
        // 1. Original Struct
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type
            ),*
        }

        // 2. Partial Struct
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        $vis struct $partial_name {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field_name : Option<$field_type>
            ),*
        }

        impl $name {
            #[allow(dead_code)]
            pub fn merge(&mut self, partial: $partial_name) {
                $(
                    if let Some(val) = partial.$field_name {
                        self.$field_name = val;
                    }
                )*
            }

            #[allow(dead_code)]
            pub fn to_partial(&self) -> $partial_name {
                $partial_name {
                    $(
                        $field_name: Some(self.$field_name.clone()),
                    )*
                }
            }

            /// Forces conversion to ObjectId for fields ending in _id or _ids
           /// Converts to BSON and recursively fixes all IDs in nested documents/arrays
            #[allow(dead_code)]
            pub fn to_document(&self) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                use mongodb::bson::{Bson, Document, oid::ObjectId};

                // Helper function to recursively fix ObjectIds anywhere in the BSON tree
                fn fix_bson_recursive(val: Bson) -> Bson {
                    match val {
                        Bson::Document(doc) => {
                            let mut new_doc = Document::new();
                            for (k, v) in doc {
                                // 1. Apply conversion logic to the current key
                                let processed_v = if k == "id" || k.ends_with("_id") {
                                    if let Bson::String(ref s) = v {
                                        ObjectId::parse_str(s).map(Bson::ObjectId).unwrap_or(v)
                                    } else { v }
                                } else if k.ends_with("_ids") {
                                    if let Bson::Array(ref arr) = v {
                                        Bson::Array(arr.iter().map(|item| {
                                            if let Bson::String(s) = item {
                                                ObjectId::parse_str(s).map(Bson::ObjectId).unwrap_or(Bson::String(s.clone()))
                                            } else { item.clone() }
                                        }).collect())
                                    } else { v }
                                } else { v };

                                // 2. Recurse deeper into the value to catch nested structures
                                new_doc.insert(k, fix_bson_recursive(processed_v));
                            }
                            Bson::Document(new_doc)
                        }
                        Bson::Array(arr) => {
                            Bson::Array(arr.into_iter().map(fix_bson_recursive).collect())
                        }
                        _ => val,
                    }
                }

                let mut final_document = Document::new();

                $(
                    {
                        #[derive(serde::Serialize)]
                        struct FieldWrapper<'a> {
                            $(#[$field_meta])*
                            pub $field_name: &'a $field_type,
                        }

                        let wrapper = FieldWrapper { $field_name: &self.$field_name };
                        let bson_val = mongodb::bson::to_bson(&wrapper)
                            .map_err(|e| $crate::errors::AppError {
                                message: format!("Serialization error for {}: {}", stringify!($field_name), e)
                            })?;
                        
                        // Process the serialized field through our recursive fixer
                        let fixed_bson = fix_bson_recursive(bson_val);

                        if let Bson::Document(doc) = fixed_bson {
                            for (k, v) in doc {
                                final_document.insert(k, v);
                            }
                        }
                    }
                )*

                Ok(final_document)
            }

            /// Used for UPDATE - Corrected to use dynamic field names
           /// Converts a partial struct to a BSON document for DATABASE operations (UPDATE)
            /// Recursively fixes IDs in nested objects or arrays
            #[allow(dead_code)]
            pub fn from_partial(partial: $partial_name) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                use mongodb::bson::{Bson, Document, oid::ObjectId};

                // Internal helper to fix ObjectIds deep in the tree
                fn fix_bson_recursive(val: Bson) -> Bson {
                    match val {
                        Bson::Document(doc) => {
                            let mut new_doc = Document::new();
                            for (k, v) in doc {
                                // Conversion logic for id, *_id, or *_ids
                                let processed_v = if k == "id" || k == "_id" || k.ends_with("_id") {
                                    if let Bson::String(ref s) = v {
                                        ObjectId::parse_str(s).map(Bson::ObjectId).unwrap_or(v)
                                    } else { v }
                                } else if k.ends_with("_ids") {
                                    if let Bson::Array(ref arr) = v {
                                        Bson::Array(arr.iter().map(|item| {
                                            if let Bson::String(s) = item {
                                                ObjectId::parse_str(s).map(Bson::ObjectId).unwrap_or(Bson::String(s.clone()))
                                            } else { item.clone() }
                                        }).collect())
                                    } else { v }
                                } else { v };

                                new_doc.insert(k, fix_bson_recursive(processed_v));
                            }
                            Bson::Document(new_doc)
                        }
                        Bson::Array(arr) => {
                            Bson::Array(arr.into_iter().map(fix_bson_recursive).collect())
                        }
                        _ => val,
                    }
                }

                let mut document = Document::new();

                $(
                    if let Some(value) = partial.$field_name {
                        #[derive(serde::Serialize)]
                        struct FieldWrapper<'a> {
                            $(#[$field_meta])*
                            pub $field_name: &'a $field_type,
                        }

                        let wrapper = FieldWrapper { $field_name: &value };
                        let bson_val = mongodb::bson::to_bson(&wrapper)
                            .map_err(|e| $crate::errors::AppError {
                                message: format!("Failed to serialize field '{}': {}", stringify!($field_name), e)
                            })?;

                        // Apply the recursive fix to the serialized field
                        let fixed_bson = fix_bson_recursive(bson_val);

                        if let Bson::Document(doc) = fixed_bson {
                            for (k, v) in doc {
                                document.insert(k, v);
                            }
                        }
                    }
                )*

                Ok(document)
            }
        }
    };
}