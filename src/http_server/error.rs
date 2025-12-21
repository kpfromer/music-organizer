use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

// A generic error report
// Produced via `Err(some_err).wrap_err("Some context")`
// or `Err(color_eyre::eyre::Report::new(SomeError))`
#[allow(dead_code)]
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Tell axum how to convert `Report` into a response.
impl IntoResponse for Report {
    fn into_response(self) -> Response<Body> {
        let err = self.0;
        let err_string = format!("{err:?}");

        log::error!("{err_string}");

        // TODO: handle errors here
        // if let Some(err) = err.downcast_ref::<DemoError>() {
        //     return err.response();
        // }

        // Fallback
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_string(),
        )
            .into_response()
    }
}
