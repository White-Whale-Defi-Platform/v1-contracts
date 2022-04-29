use serde::{Serialize, Serializer};

pub fn serde_option<T>(option: Option<T>) -> String
where
    T: ToString,
{
    match option {
        Some(v) => v.to_string(),
        None => "none".to_string(),
    }
}
