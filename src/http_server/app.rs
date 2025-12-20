use std::{path::PathBuf, sync::Arc};

use async_graphql_axum::GraphQL;
use axum::{Router, routing::get};
use color_eyre::eyre::{Context, eyre};
use tower::ServiceBuilder;
#[cfg(not(debug_assertions))]
use tower_http::cors::AllowMethods;
use tower_http::cors::CorsLayer;

use crate::{
    config::Config,
    database::Database,
    http_server::{graphql, state::AppState},
    import_track::watch_directory,
};

async fn root() -> &'static str {
    "Hello, World!"
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
        let _r = tokio::spawn(async move {
            // TODO: handle errors hear
            // maybe restart 5 times with a delay between each restart
            // if it fails after 5 restarts, log an error and exit?
            let _ = watch_directory(
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
