use async_graphql::{Error, ErrorExtensions};

#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphqlError {
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Failed to get app state")]
    FailedToGetAppState,
}

impl Default for GraphqlError {
    fn default() -> Self {
        Self::ServerError("Unknown error".to_string())
    }
}

impl From<color_eyre::Report> for GraphqlError {
    fn from(report: color_eyre::Report) -> Self {
        // Log the full error report with trace chain for debugging
        log::error!("GraphQL error: {:#?}", report);
        Self::ServerError(report.to_string())
    }
}

impl ErrorExtensions for GraphqlError {
    fn extend(&self) -> Error {
        Error::new(format!("{}", self)).extend_with(|_err, e| match self {
            GraphqlError::ServerError(reason) => e.set("reason", reason.clone()),
            GraphqlError::FailedToGetAppState => {
                e.set("reason", "Failed to get app state".to_string())
            }
        })
    }
}

// Newtype wrapper to avoid blanket From implementation conflict for GraphqlError and async_graphql::Error
#[derive(Debug, Clone)]
pub struct GraphqlErrorWrapper(GraphqlError);

impl From<GraphqlError> for GraphqlErrorWrapper {
    fn from(err: GraphqlError) -> Self {
        Self(err)
    }
}

impl From<GraphqlErrorWrapper> for Error {
    fn from(wrapper: GraphqlErrorWrapper) -> Self {
        wrapper.0.extend()
    }
}

// Make it easy to convert from color_eyre::Report
impl From<color_eyre::Report> for GraphqlErrorWrapper {
    fn from(report: color_eyre::Report) -> Self {
        GraphqlError::from(report).into()
    }
}

pub type GraphqlResult<T> = Result<T, GraphqlErrorWrapper>;
