use tracing::{info, instrument};

#[utoipa::path(
    get,
    path = "/items",
    responses(
        (status = 200, description = "item", body = String)
    ),
    tag = "Test"
)]
#[instrument(name = "GET /items")]
pub async fn items_handler() -> &'static str {
    info!("Items route was called!");
    r#"["item1", "item2"]"#
}
