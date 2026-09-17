use tracing::{info, instrument};

#[instrument(name = "GET /items")]
pub async fn items_handler() -> &'static str {
    info!("Items route was called!");
    r#"["item1", "item2"]"#
}
