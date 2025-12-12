use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_graphql_axum::GraphQL;
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    handler::Handler,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use axum_macros::debug_handler;
use color_eyre::eyre::{Context, eyre};
use log::log;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
#[cfg(debug_assertions)]
use tower_http::cors::AllowMethods;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    config::Config,
    database::{Database, Track},
    http_server::{graphql, state::AppState},
    import_track::watch_directory,
};

pub type Result<T, E = Report> = color_eyre::Result<T, E>;
// A generic error report
// Produced via `Err(some_err).wrap_err("Some context")`
// or `Err(color_eyre::eyre::Report::new(SomeError))`
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Tell axum how to convert `Report` into a response.
impl IntoResponse for Report {
    fn into_response(self) -> Response<Body> {
        let err = self.0;
        let err_string = format!("{err:?}");

        log::error!("{err_string}");

        // if let Some(err) = err.downcast_ref::<DemoError>() {
        //     return err.response();
        // }

        // Fallback
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_string(),
        )
            .into_response()
    }
}
// TODO: extract above to a module

// TODO: move
// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
async fn tracks(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Track>>> {
    Ok(Json(state.db.get_tracks().await?))
}

pub async fn start(
    port: u16,
    database: Database,
    config: Config,
    acoustid_api_key: &str,
    watch_directory_path: PathBuf,
) -> color_eyre::Result<()> {
    let app_state = Arc::new(AppState { db: database });

    let schema = graphql::create_schema(app_state.clone());

    #[cfg(debug_assertions)]
    let cors_layer = CorsLayer::permissive();

    #[cfg(not(debug_assertions))]
    let cors_layer = CorsLayer::new()
        .allow_origin(Origin::exact("https://your-production-domain.com")) // TODO: use env variable
        .allow_methods(AllowMethods::any());

    let app = Router::new()
        .route("/", get(root))
        .route("/tracks", get(tracks))
        .route(
            "/graphql",
            get(graphql::graphql).post_service(GraphQL::new(schema)),
        )
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(app_state.clone());

    // Start watch directory in background
    {
        log::info!("Watching directory in background");
        let watch_directory_path = watch_directory_path.clone();
        let acoustid_api_key = acoustid_api_key.to_string();
        let r = tokio::spawn(async move {
            // TODO: handle errors hear
            // maybe restart 5 times with a delay between each restart
            // if it fails after 5 restarts, log an error and exit?
            watch_directory(
                &watch_directory_path,
                &acoustid_api_key,
                &config,
                &app_state.db,
            )
            .await;
        });
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .wrap_err_with(|| eyre!("Failed to bind to port {}", port))?;
    axum::serve(listener, app)
        .await
        .wrap_err("Failed to start HTTP server")?;

    Ok(())
}
