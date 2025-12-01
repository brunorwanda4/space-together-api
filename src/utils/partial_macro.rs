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
        // --- 1. Original Struct ---
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type
            ),*
        }

        // --- 2. Partial Struct ---
        // FIX 1: Add Serialize and Deserialize here
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
        $vis struct $partial_name {
            $(
                // FIX 2: Skip serializing fields that are None.
                // This prevents your database update from setting missing fields to "null".
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
        }
    };
}
