use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{Instrument, info_span}; // Bring Instrument trait into scope
use uuid::Uuid;

use crate::error::AppError;

#[derive(Deserialize)]
pub struct CreateItem {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

use axum::extract::Query;

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tracing::instrument(
    name = "http.post.create_item",
    skip(pool,payload), 
    fields(item.name = %payload.name)
)]
pub async fn create_item(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), AppError> {
    let item = sqlx::query_as!(
        Item,
        r#"
        INSERT INTO items (id, name, description)
        VALUES ($1, $2, $3)
        RETURNING id, name, description
        "#,
        Uuid::new_v4(),
        payload.name,
        payload.description
    )
    .fetch_one(&pool)
    .instrument(info_span!("db.query.insert_item"))
    .await?; // Automatically converted to AppError if it fails

    Ok((StatusCode::CREATED, Json(item)))
}

#[tracing::instrument(
    name = "http.get.list_items",
    skip(pool, pagination),
    fields(
        db.limit = pagination.limit.unwrap_or(10),
        db.offset = pagination.offset.unwrap_or(0)
    )
)]
pub async fn list_items(
    State(pool): State<PgPool>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<Item>>, AppError> {
    
    // Default to 10 items. Clamp ensures the user cannot request less than 1 
    // or more than 100 items per request, preventing memory exhaustion.
    let limit = pagination.limit.unwrap_or(10).clamp(1, 100);
    
    // Default to offset 0. Max(0) prevents negative offsets from crashing the DB.
    let offset = pagination.offset.unwrap_or(0).max(0);

    let items = sqlx::query_as!(
        Item,
        r#"
        SELECT id, name, description
        FROM items
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&pool)
    .instrument(info_span!("db.query.select_items"))
    .await?;

    Ok(Json(items))
}
