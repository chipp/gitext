use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub id: u16,
    pub status: PipelineStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Pending,
    Running,
    Success,
    Failed,
}
