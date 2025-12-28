use std::path::PathBuf;
use std::sync::Arc;

use async_graphql_axum::GraphQL;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use color_eyre::eyre::{Context, eyre};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{
    config::Config,
    database::Database,
    http_server::{
        graphql, http_routes::album_art_image::get_track_album_art_image, state::AppState,
    },
    import_track::watch_directory,
    soulseek::{SearchConfig, SoulSeekClientContext},
};

// Handler to serve index.html for SPA routing
async fn serve_index(frontend_dist: PathBuf) -> Response {
    let index_path = frontend_dist.join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

pub struct HttpServerConfig {
    pub port: u16,
    pub database: Database,
    pub config: Config,
    pub acoustid_api_key: String,
    pub watch_directory_path: PathBuf,
    pub soulseek_username: String,
    pub soulseek_password: String,
    pub download_directory: PathBuf,
}

pub async fn start(config: HttpServerConfig) -> color_eyre::Result<()> {
    let HttpServerConfig {
        port,
        database,
        config,
        acoustid_api_key,
        watch_directory_path,
        soulseek_username,
        soulseek_password,
        download_directory,
    } = config;
    log::info!("Initializing SoulSeek client context");
    let soulseek_context = SoulSeekClientContext::new(SearchConfig {
        username: soulseek_username.to_string(),
        password: soulseek_password.to_string(),
        concurrency: Some(2),
        searches_per_time: Some(34),
        renew_time_secs: Some(220),
        max_search_time_ms: Some(8000),
        remove_special_chars: Some(true),
    })
    .await
    .wrap_err("Failed to initialize SoulSeek client context")?;

    let app_state = Arc::new(AppState {
        db: database,
        soulseek_context,
        download_directory,
        api_key: acoustid_api_key.clone(),
        config: config.clone(),
    });

    let schema = graphql::create_schema(app_state.clone());

    let cors_layer = CorsLayer::permissive();

    // Check if we're in release mode (production)
    let is_release = cfg!(not(debug_assertions));

    // Validate frontend files in release mode
    if is_release {
        let frontend_dist = PathBuf::from("frontend/dist");
        let index_html_path = frontend_dist.join("index.html");

        // Validate that frontend files exist
        if !index_html_path.exists() {
            return Err(eyre!(
                "Release mode requires frontend/dist/index.html but it was not found at: {}",
                index_html_path.display()
            ));
        }

        // Check that dist directory exists and has some files
        if !frontend_dist.exists() {
            return Err(eyre!(
                "Release mode requires frontend/dist directory but it was not found at: {}",
                frontend_dist.display()
            ));
        }

        // Check for JS or CSS files in dist (Bun outputs these directly in dist)
        let dist_entries = std::fs::read_dir(&frontend_dist)
            .map_err(|e| eyre!("Failed to read frontend/dist directory: {}", e))?;
        let has_assets = dist_entries.filter_map(|entry| entry.ok()).any(|entry| {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str());
            file_name
                .map(|n| n.ends_with(".js") || n.ends_with(".css"))
                .unwrap_or(false)
        });

        if !has_assets {
            return Err(eyre!(
                "Release mode requires frontend/dist to contain JS or CSS files, but none were found at: {}",
                frontend_dist.display()
            ));
        }

        log::info!(
            "Release mode: Serving static files from: {}",
            frontend_dist.display()
        );
    }

    // Build router with API routes first
    let mut app = Router::new().route(
        "/graphql",
        get(graphql::graphql).post_service(GraphQL::new(schema)),
    );

    // In release mode, serve static files and index.html via fallback
    // ServeDir handles static files, and falls back to index.html for SPA routing
    if is_release {
        let frontend_dist = PathBuf::from("frontend/dist");
        let dist_clone = frontend_dist.clone();
        app = app.fallback_service(ServeDir::new(&frontend_dist).fallback(get(move || {
            let dist = dist_clone.clone();
            async move { serve_index(dist).await }
        })));
    }

    let app = app
        .route(
            "/album-art-image/{track_id}",
            get(get_track_album_art_image),
        )
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(app_state.clone());

    // Start watch directory in background
    {
        log::info!("Watching directory in background");
        let watch_directory_path = watch_directory_path.clone();
        let app_state_clone = app_state.clone();
        let _r = tokio::spawn(async move {
            // TODO: handle errors hear
            // maybe restart 5 times with a delay between each restart
            // if it fails after 5 restarts, log an error and exit?
            match watch_directory(
                &watch_directory_path,
                &app_state_clone.api_key,
                &app_state_clone.config,
                &app_state_clone.db,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error watching directory: {}", e);
                }
            }
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
