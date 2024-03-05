use axum::{
    extract::State,
    http::Method,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use orgize::Org;
use std::{
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tower_http::cors::{Any, CorsLayer};

use crate::common::search_heading::SearchOption;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        eprintln!("{:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

type ApiState = Arc<RwLock<Vec<(PathBuf, Org)>>>;

pub async fn start(path: Vec<PathBuf>) -> anyhow::Result<()> {
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    eprintln!("Listening at {addr:?}");

    let mut state = Vec::with_capacity(path.len());

    for path in path {
        match fs::read_to_string(&path) {
            Ok(content) => {
                state.push((path, Org::parse(content)));
            }
            Err(err) => {
                eprintln!("Failed to read {}: {:?}", path.display(), err);
            }
        }
    }

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/search-headline", post(search_heading))
        .with_state(Arc::new(RwLock::new(state)))
        .layer(cors);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn search_heading(
    State(state): State<ApiState>,
    Json(body): Json<SearchOption>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.read().unwrap();

    let mut results = vec![];
    for (_, org) in state.iter() {
        results.append(&mut crate::common::search_heading::search(&body, &org));
    }

    Ok(Json(results))
}
