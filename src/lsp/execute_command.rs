use lsp_types::{ExecuteCommandParams, MessageType};
use serde_json::Value;

use crate::{base::Server, command::OrgwiseCommand};

pub async fn execute_command<S: Server>(s: &S, params: ExecuteCommandParams) -> Option<Value> {
    let ExecuteCommandParams {
        command, arguments, ..
    } = params;

    match OrgwiseCommand::try_from((command.as_str(), arguments)) {
        Ok(cmd) => match cmd.execute(s).await {
            Ok(value) => Some(value),
            Err(err) => {
                s.show_message(
                    MessageType::ERROR,
                    format!("Failed to execute {command:?}: {err}"),
                )
                .await;

                None
            }
        },
        Err(_) => {
            s.show_message(MessageType::ERROR, format!("Unknown command {command:?}"))
                .await;

            None
        }
    }
}
