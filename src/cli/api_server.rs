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
use crate::{backend::Backend, cli::environment::CliBackend};

#[derive(Debug, Args)]
pub struct Command {
    #[arg(short, long)]
    port: Option<u16>,
    path: Vec<PathBuf>,
}

type AppState = Arc<CliBackend>;

impl Command {
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            self.port.unwrap_or(3000),
        ));

        log::info!("Listening at {addr:?}");

        let backend = CliBackend::new(false);

        for path in &self.path {
            backend.load_org_file(path);
        }

        log::info!("Loaded {} org file(s)", backend.documents().len());

        let state = AppState::new(backend);

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
) -> Response {
    command
        .execute_response(state.as_ref())
        .await
        .unwrap_or_else(|err| {
            log::error!("{err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {err}"),
            )
                .into_response()
        })
}
