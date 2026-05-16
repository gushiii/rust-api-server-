use serde_json::Value;
use sqlx::{MySql, mysql::MySqlArguments, query::Query};

pub fn bind_json_value<'q>(
    query: Query<'q, MySql, MySqlArguments>,
    json_val: &Value,
) -> Query<'q, MySql, MySqlArguments> {
    match json_val {
        Value::String(s) => {
            if let Ok(i) = s.parse::<i64>() {
                query.bind(i)
            } else if let Ok(f) = s.parse::<f64>() {
                query.bind(f)
            } else {
                query.bind(s.clone())
            }
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else {
                query.bind(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::Bool(b) => query.bind(*b),
        Value::Null => query.bind(None::<String>),
        _ => query.bind(json_val.to_string()),
    }
}
