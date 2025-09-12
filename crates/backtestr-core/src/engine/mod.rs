use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTFEngine {
    pub name: String,
}

impl MTFEngine {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Default for MTFEngine {
    fn default() -> Self {
        Self::new("BackTestr MTF Engine".to_string())
    }
}
