use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{routes, state::AppState};

mod health;
mod items;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_handler,
        // Add other annotated routes here
    ),
    tags(
        (name = "Health", description = "Service health checks")
    )
)]
pub struct ApiDoc;

pub fn create_router(is_prod: bool) -> Router<AppState> {
    let mut router = Router::new()
        .route("/healthz", get(health::health_handler))
        .route("/items", post(items::create_item))
        .route("/items", get(items::list_items));

    if !is_prod {
        let swagger =
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", routes::ApiDoc::openapi());
        router = router.merge(swagger);
    }
    router
}
