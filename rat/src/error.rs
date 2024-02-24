pub enum RatError {
    Unauthorized,
    Error(anyhow::Error),
}

impl From<anyhow::Error> for RatError {
    fn from(error: anyhow::Error) -> Self {
        RatError::Error(error)
    }
}
