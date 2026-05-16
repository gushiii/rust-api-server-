use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::binder::bind_json_value;
use crate::encoder::mysql_row_to_json;
use crate::parser::{parse_query_params, validate_identifier};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::MySqlPool,
}

async fn get_primary_key(pool: &sqlx::MySqlPool, table_name: &str) -> Result<String, String> {
    let sql = "SELECT COLUMN_NAME FROM information_schema.KEY_COLUMN_USAGE \
               WHERE TABLE_SCHEMA = DATABASE() AND CONSTRAINT_NAME = 'PRIMARY' AND TABLE_NAME = ? LIMIT 1";
    let row_opt = sqlx::query_scalar::<_, String>(sql)
        .bind(table_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row_opt.unwrap_or_else(|| "id".to_string()))
}

pub async fn handle_create(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, String> {
    validate_identifier(&table_name)?;
    let obj = payload.as_object().ok_or("Payload must be a JSON object")?;

    let mut columns = Vec::new();
    let mut placeholders = Vec::new();
    for (key, _) in obj {
        validate_identifier(key)?;
        columns.push(format!("`{}`", key));
        placeholders.push("?");
    }

    let sql = format!(
        "INSERT INTO `{}` ({}) VALUES ({})",
        table_name,
        columns.join(", "),
        placeholders.join(", ")
    );
    let mut query = sqlx::query(&sql);
    for (_, val) in obj {
        query = bind_json_value(query, val);
    }
    let result = query
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let pk = get_primary_key(&state.pool, &table_name).await?;
    let select_sql = format!("SELECT * FROM `{}` WHERE `{}` = ?", table_name, pk);
    let row = if let Some(front_pk) = obj.get(&pk) {
        bind_json_value(sqlx::query(&select_sql), front_pk)
            .fetch_one(&state.pool)
            .await
    } else {
        sqlx::query(&select_sql)
            .bind(result.last_insert_id())
            .fetch_one(&state.pool)
            .await
    }
    .map_err(|e| e.to_string())?;

    Ok(Json(mysql_row_to_json(&row)))
}

pub async fn handle_list(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, String> {
    validate_identifier(&table_name)?;

    let ctx = parse_query_params(&params)?;

    let mut sql = format!(
        "SELECT * FROM `{}` WHERE 1=1 {}",
        table_name, ctx.sql_clauses
    );
    if ctx.limit.is_some() {
        sql.push_str(" LIMIT ?");
    }
    if ctx.offset.is_some() {
        sql.push_str(" OFFSET ?");
    }

    let mut query = sqlx::query(&sql);
    for val in &ctx.bind_values {
        query = bind_json_value(query, val);
    }
    if let Some(l) = ctx.limit {
        query = query.bind(l);
    }
    if let Some(o) = ctx.offset {
        query = query.bind(o);
    }

    let rows = query
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(Value::Array(
        rows.iter().map(mysql_row_to_json).collect(),
    )))
}

pub async fn handle_get(
    State(state): State<AppState>,
    Path((table_name, id)): Path<(String, String)>,
) -> Result<Json<Value>, String> {
    validate_identifier(&table_name)?;
    let pk = get_primary_key(&state.pool, &table_name).await?;
    let row = sqlx::query(&format!(
        "SELECT * FROM `{}` WHERE `{}` = ?",
        table_name, pk
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(Json(mysql_row_to_json(&row)))
}

pub async fn handle_update(
    State(state): State<AppState>,
    Path((table_name, id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, String> {
    validate_identifier(&table_name)?;
    let obj = payload.as_object().ok_or("Payload must be a JSON object")?;
    let pk = get_primary_key(&state.pool, &table_name).await?;

    let mut set_clauses = Vec::new();
    let mut query_values = Vec::new();
    for (key, value) in obj {
        validate_identifier(key)?;
        if key == &pk {
            continue;
        }
        set_clauses.push(format!("`{}` = ?", key));
        query_values.push(value);
    }

    if set_clauses.is_empty() {
        return Err("No fields provided for update".to_string());
    }

    let sql = format!(
        "UPDATE `{}` SET {} WHERE `{}` = ?",
        table_name,
        set_clauses.join(", "),
        pk
    );

    let mut query = sqlx::query(&sql);
    for val in query_values {
        query = bind_json_value(query, val);
    }

    query
        .bind(id.clone())
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let row = sqlx::query(&format!(
        "SELECT * FROM `{}` WHERE `{}` = ?",
        table_name, pk
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(mysql_row_to_json(&row)))
}

pub async fn handle_delete(
    State(state): State<AppState>,
    Path((table_name, id)): Path<(String, String)>,
) -> Result<Json<Value>, String> {
    validate_identifier(&table_name)?;
    let pk = get_primary_key(&state.pool, &table_name).await?;
    let result = sqlx::query(&format!("DELETE FROM `{}` WHERE `{}` = ?", table_name, pk))
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut response = Map::new();
    response.insert("success".to_string(), Value::Bool(true));
    response.insert(
        "rows_affected".to_string(),
        Value::Number(result.rows_affected().into()),
    );
    Ok(Json(Value::Object(response)))
}
