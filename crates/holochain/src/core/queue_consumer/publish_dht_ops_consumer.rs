//! The workflow and queue consumer for sys validation

use super::*;
use crate::core::{
    state::workspace::Workspace,
    workflow::publish_dht_ops_workflow::{publish_dht_ops_workflow, PublishDhtOpsWorkspace},
};
use futures::StreamExt;
use holochain_state::env::EnvironmentWrite;
use holochain_state::env::ReadManager;

/// Spawn the QueueConsumer for Publish workflow
pub fn spawn_publish_dht_ops_consumer(
    env: EnvironmentWrite,
) -> (QueueTrigger, tokio::sync::oneshot::Receiver<()>) {
    let (tx, mut rx) = QueueTrigger::new();
    let (tx_first, rx_first) = tokio::sync::oneshot::channel();
    let mut tx_first = Some(tx_first);
    let mut trigger_self = tx.clone();
    let _handle = tokio::spawn(async move {
        loop {
            let env_ref = env.guard().await;
            let reader = env_ref.reader().expect("Could not create LMDB reader");
            let workspace =
                PublishDhtOpsWorkspace::new(&reader, &env_ref).expect("Could not create Workspace");
            if let WorkComplete::Incomplete =
                publish_dht_ops_workflow(workspace, env.clone().into())
                    .await
                    .expect("Error running Workflow")
            {
                trigger_self.trigger().expect("Trigger channel closed")
            };
            if let Some(mut tx_first) = tx_first.take() {
                let _ = tx_first.send(());
            }
            rx.next().await;
        }
    });
    (tx, rx_first)
}
