use crate::actor::actor_config::RestartPolicy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub name: String,
    pub default_mailbox_size: usize,
    pub default_message_throughput: usize,
    pub default_restart_policy: RestartPolicy,
}
