use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BuildStats {
    pub cancelled: Option<u32>,
    pub successful: Option<u32>,
    pub in_progress: Option<u32>,
    pub failed: Option<u32>,
    pub unknown: Option<u32>,
}

pub enum MergedBuildStatus {
    Success,
    InProgress,
    Failed,
}

impl From<BuildStats> for MergedBuildStatus {
    fn from(value: BuildStats) -> Self {
        if value.failed.unwrap_or_default() > 0
            || value.cancelled.unwrap_or_default() > 0
            || value.unknown.unwrap_or_default() > 0
        {
            Self::Failed
        } else if value.in_progress.unwrap_or_default() > 0 {
            Self::InProgress
        } else if value.successful.unwrap_or_default() > 0 {
            Self::Success
        } else {
            Self::Failed
        }
    }
}
