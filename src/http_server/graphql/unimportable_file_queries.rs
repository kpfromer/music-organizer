use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::entities::unimportable_file::UnimportableReason;

#[derive(Debug, Clone, SimpleObject)]
pub struct UnimportableFile {
    pub id: i64,
    pub file_path: String,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
    pub reason: UnimportableReason,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UnimportableFilesResponse {
    pub files: Vec<UnimportableFile>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}
