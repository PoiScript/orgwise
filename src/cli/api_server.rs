use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use clap::Args;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::Arc,
};
use tower_http::cors::{Any, CorsLayer};

use crate::command::OrgwiseCommand;
use crate::{base::Server, cli::environment::CliServer};

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        log::error!("{:?}", self.0);
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

#[derive(Debug, Args)]
pub struct Command {
    #[arg(short, long)]
    port: Option<u16>,
    path: Vec<PathBuf>,
}

type AppState = Arc<CliServer>;

impl Command {
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            self.port.unwrap_or(3000),
        ));

        log::info!("Listening at {addr:?}");

        let base = CliServer::new(false);

        for path in &self.path {
            base.load_org_file(path);
        }

        log::info!("Loaded {} org file(s)", base.documents().len());

        let state = AppState::new(base);

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
            .allow_origin(Any);

        let app = Router::new()
            .route("/api/command", post(execute_command))
            .with_state(state)
            .layer(cors);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

async fn execute_command(
    State(state): State<AppState>,
    Json(command): Json<OrgwiseCommand>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(command.execute(state.as_ref()).await?))
}
