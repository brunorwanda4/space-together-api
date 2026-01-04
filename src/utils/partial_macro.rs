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
        // --- 1. Original Struct (Keep as is for API/JSON) ---
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type
            ),*
        }

        // --- 2. Partial Struct (For Updates AND Creates) ---
        // We STRIP $(#[$field_meta])* intentionally to bypass object_id_helpers
        // This ensures MongoDB saves them as actual ObjectIds.
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        $vis struct $partial_name {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field_name : Option<$field_type>
            ),*
        }

        // --- 3. Implementation ---
        impl $name {
            #[allow(dead_code)]
            pub fn merge(&mut self, partial: $partial_name) {
                $(
                    if let Some(val) = partial.$field_name {
                        self.$field_name = val;
                    }
                )*
            }

            // NEW: Helper to convert Main to Partial for DB Saving
            #[allow(dead_code)]
            pub fn to_partial(&self) -> $partial_name {
                $partial_name {
                    $(
                        $field_name: Some(self.$field_name.clone()),
                    )*
                }
            }
        }
    };
}
