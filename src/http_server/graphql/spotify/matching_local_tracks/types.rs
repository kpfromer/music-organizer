#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<i32>,
}
