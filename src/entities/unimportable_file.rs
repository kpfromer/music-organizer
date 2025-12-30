use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(SimpleObject, Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[graphql(name = "UnimportableFile")]
#[sea_orm(table_name = "unimportable_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub file_path: String,
    pub sha256: String,
    pub created_at: i64,
    pub reason: UnimportableReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, async_graphql::Enum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[graphql(name = "UnimportableReason")]
pub enum UnimportableReason {
    #[sea_orm(string_value = "unsupported_file_type")]
    UnsupportedFileType,
    #[sea_orm(string_value = "duplicate_track")]
    DuplicateTrack,
    #[sea_orm(string_value = "file_system_error")]
    FileSystemError,
    #[sea_orm(string_value = "hash_computation_error")]
    HashComputationError,
    #[sea_orm(string_value = "chromaprint_error")]
    ChromaprintError,
    #[sea_orm(string_value = "acoust_id_error")]
    AcoustIdError,
    #[sea_orm(string_value = "musicbrainz_error")]
    MusicBrainzError,
    #[sea_orm(string_value = "database_error")]
    DatabaseError,
}

// Conversion from ImportError to UnimportableReason
impl From<&crate::import_track::ImportError> for UnimportableReason {
    fn from(error: &crate::import_track::ImportError) -> Self {
        use crate::import_track::ImportError;
        match error {
            ImportError::UnsupportedFileType { .. } => UnimportableReason::UnsupportedFileType,
            ImportError::DuplicateTrack { .. } => UnimportableReason::DuplicateTrack,
            ImportError::FileSystemError { .. } => UnimportableReason::FileSystemError,
            ImportError::HashComputationError { .. } => UnimportableReason::HashComputationError,
            ImportError::ChromaprintError { .. } => UnimportableReason::ChromaprintError,
            ImportError::AcoustIdError { .. } => UnimportableReason::AcoustIdError,
            ImportError::MusicBrainzError { .. } => UnimportableReason::MusicBrainzError,
            ImportError::DatabaseError { .. } => UnimportableReason::DatabaseError,
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
