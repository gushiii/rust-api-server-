use serde_json::Value;
use std::collections::HashMap;

pub struct QueryContext {
    pub select_fields: String,
    pub sql_clauses: String,
    pub bind_values: Vec<Value>,
    pub group_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn validate_identifier(name: &str) -> Result<(), String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "Security error: Invalid database identifier '{}'",
            name
        ));
    }
    Ok(())
}

pub fn parse_query_params(params: &HashMap<String, String>) -> Result<QueryContext, String> {
    let mut sql_clauses = String::new();
    let mut bind_values = Vec::new();
    let mut limit = None;
    let mut offset = None;
    let mut sort_by = None;
    let mut order_by = "ASC".to_string();

    let mut select_list = Vec::new();
    let mut group_by = None;

    if let Some(g_str) = params.get("_group") {
        validate_identifier(g_str)?;
        group_by = Some(g_str.to_string());

        select_list.push(format!("`{}`", g_str));
    }
    if let Some(l_str) = params.get("_limit") {
        limit = l_str.parse::<i64>().ok();
    }
    if let Some(o_str) = params.get("_offset") {
        offset = o_str.parse::<i64>().ok();
    }

    for (key, value) in params {
        if key == "_limit"
            || key == "_offset"
            || key == "_where"
            || key == "_group"
            || key == "_aggregate"
        {
            continue;
        }
        validate_identifier(key)?;
        sql_clauses.push_str(&format!(" AND `{}` = ?", key));
        bind_values.push(Value::String(value.clone()));
    }

    if let Some(where_str) = params.get("_where") {
        let where_obj: Value = serde_json::from_str(where_str)
            .map_err(|e| format!("Invalid JSON inside _where parameter: {}", e))?;

        if let Some(conditions) = where_obj.as_object() {
            for (field, block) in conditions {
                if field == "_sort" {
                    if let Some(s) = block.as_str() {
                        validate_identifier(s)?;
                        sort_by = Some(s.to_string());
                    }
                    continue;
                }
                if field == "_order" {
                    if let Some(o) = block.as_str() {
                        let o_upper = o.to_uppercase();
                        if o_upper == "ASC" || o_upper == "DESC" {
                            order_by = o_upper;
                        } else {
                            return Err("Invalid _order value. Must be 'asc' or 'desc'".to_string());
                        }
                    }
                    continue;
                }

                validate_identifier(field)?;

                match block {
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                        sql_clauses.push_str(&format!(" AND `{}` = ?", field));
                        bind_values.push(block.clone());
                    }
                    Value::Object(inner_map) => {
                        for (op, op_val) in inner_map {
                            let op_sql = match op.as_str() {
                                "$gt" => ">",
                                "$gte" => ">=",
                                "$lt" => "<",
                                "$lte" => "<=",
                                "$neq" => "!=",
                                "$like" => "LIKE",
                                _ => return Err(format!("Unsupported operator: {}", op)),
                            };
                            sql_clauses.push_str(&format!(" AND `{}` {} ?", field, op_sql));

                            if op.as_str() == "$like" {
                                let raw_str = op_val.as_str().unwrap_or("");
                                bind_values.push(Value::String(format!("%{}%", raw_str)));
                            } else {
                                bind_values.push(op_val.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(agg_str) = params.get("_aggregate") {
        let agg_obj: Value = serde_json::from_str(agg_str)
            .map_err(|e| format!("Invalid JSON inside _aggregate parameter: {}", e))?;

        if let Some(agg_map) = agg_obj.as_object() {
            for (func_type, target_field) in agg_map {
                let field_str = target_field.as_str().unwrap_or("*");
                if field_str != "*" {
                    validate_identifier(field_str)?;
                }

                match func_type.as_str() {
                    "count" => {
                        if field_str == "*" {
                            select_list.push("COUNT(*) AS `count`".to_string());
                        } else {
                            select_list
                                .push(format!("COUNT(`{}`) AS `count_{}`", field_str, field_str));
                        }
                    }
                    "sum" => {
                        select_list.push(format!("SUM(`{}`) AS `sum_{}`", field_str, field_str))
                    }
                    "avg" => {
                        select_list.push(format!("AVG(`{}`) AS `avg_{}`", field_str, field_str))
                    }
                    "max" => {
                        select_list.push(format!("MAX(`{}`) AS `max_{}`", field_str, field_str))
                    }
                    "min" => {
                        select_list.push(format!("MIN(`{}`) AS `min_{}`", field_str, field_str))
                    }
                    _ => return Err(format!("Unsupported aggregate function: {}", func_type)),
                }
            }
        }
    }

    let select_fields = if select_list.is_empty() {
        "*".to_string()
    } else {
        select_list.join(", ")
    };

    if let Some(sort_field) = sort_by {
        sql_clauses.push_str(&format!(" ORDER BY `{}` {}", sort_field, order_by));
    }

    Ok(QueryContext {
        select_fields,
        sql_clauses,
        bind_values,
        group_by,
        limit,
        offset,
    })
}
