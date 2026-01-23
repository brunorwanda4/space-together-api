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
            // SHARED HELPER: Recursive BSON Fixer
            fn _fix_bson_recursive(val: mongodb::bson::Bson) -> mongodb::bson::Bson {
                use mongodb::bson::{Bson, Document, oid::ObjectId};
                match val {
                    Bson::Document(doc) => {
                        let mut new_doc = Document::new();
                        for (k, v) in doc {
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
                            new_doc.insert(k, Self::_fix_bson_recursive(processed_v));
                        }
                        Bson::Document(new_doc)
                    }
                    Bson::Array(arr) => {
                        Bson::Array(arr.into_iter().map(Self::_fix_bson_recursive).collect())
                    }
                    _ => val,
                }
            }

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

            #[allow(dead_code)]
            pub fn to_document(&self) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                let mut final_doc = mongodb::bson::Document::new();
                $(
                    {
                        #[derive(serde::Serialize)]
                        struct FieldWrapper<'a> {
                            $(#[$field_meta])*
                            pub $field_name: &'a $field_type,
                        }
                        let bson_val = mongodb::bson::to_bson(&FieldWrapper { $field_name: &self.$field_name })
                            .map_err(|e| $crate::errors::AppError {
                                message: format!("Serialization error for {}: {}", stringify!($field_name), e)
                            })?;
                        
                        if let mongodb::bson::Bson::Document(doc) = Self::_fix_bson_recursive(bson_val) {
                            for (k, v) in doc { final_doc.insert(k, v); }
                        }
                    }
                )*
                Ok(final_doc)
            }

            #[allow(dead_code)]
            pub fn from_partial(partial: $partial_name) -> Result<mongodb::bson::Document, $crate::errors::AppError> {
                let mut final_doc = mongodb::bson::Document::new();
                $(
                    if let Some(value) = partial.$field_name {
                        #[derive(serde::Serialize)]
                        struct FieldWrapper<'a> {
                            $(#[$field_meta])*
                            pub $field_name: &'a $field_type,
                        }
                        let bson_val = mongodb::bson::to_bson(&FieldWrapper { $field_name: &value })
                            .map_err(|e| $crate::errors::AppError {
                                message: format!("Failed to serialize field '{}': {}", stringify!($field_name), e)
                            })?;

                        if let mongodb::bson::Bson::Document(doc) = Self::_fix_bson_recursive(bson_val) {
                            for (k, v) in doc { final_doc.insert(k, v); }
                        }
                    }
                )*
                Ok(final_doc)
            }
        }
    };
}