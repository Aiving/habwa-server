use std::collections::HashMap;

pub(crate) enum Value {
    Integer(u32),
    String(String),
    Boolean(bool),
    None,
}

impl Value {
    pub(crate) fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Integer(integer) => Some(*integer),
            _ => None,
        }
    }

    pub(crate) fn as_string(&self) -> Option<String> {
        match self {
            Value::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }
}

pub(crate) fn parse<T: Into<String>>(data: T) -> HashMap<String, Value> {
    let data: String = data.into();
    let mut table = HashMap::new();

    for line in data.split('\n') {
        if line.starts_with('#') {
            continue;
        }

        let arr: Vec<&str> = line.split('=').collect();

        let key = arr[0].to_string();
        let value = if arr.len() > 1 {
            Some(arr[1].to_string())
        } else {
            None
        };

        if let Some(value) = value {
            if value.trim() == "" {
                table.insert(key, Value::None);
            } else if let Ok(integer) = value.parse() {
                table.insert(key, Value::Integer(integer));
            } else if let Ok(boolean) = value.parse() {
                table.insert(key, Value::Boolean(boolean));
            } else {
                table.insert(key, Value::String(value));
            }
        } else {
            table.insert(key, Value::None);
        }
    }

    table
}
