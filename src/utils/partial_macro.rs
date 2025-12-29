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
                pub $field_name : Option<$crate::strip_option!($field_type)>
            ),*
        }

        // -------- Implementation --------
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
        }
    };
}
