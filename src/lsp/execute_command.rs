use lsp_types::{ExecuteCommandParams, MessageType};
use serde_json::Value;

use crate::{backend::Backend, command::OrgwiseCommand};

pub async fn execute_command<B: Backend>(
    backend: &B,
    mut params: ExecuteCommandParams,
) -> Option<Value> {
    let name = params.command.as_str().strip_prefix("orgwise.")?;
    let argument = params.arguments.pop()?;

    let Some(cmd) = OrgwiseCommand::from_value(name, argument) else {
        backend
            .show_message(MessageType::WARNING, format!("Unknown command {name:?}"))
            .await;
        return None;
    };

    match cmd.execute(backend).await {
        Ok(value) => Some(value),
        Err(err) => {
            backend
                .show_message(
                    MessageType::ERROR,
                    format!("Failed to execute {name:?}: {err}"),
                )
                .await;

            None
        }
    }
}
