use std::fmt::{Debug, Display, Formatter};

// pub struct C2Error {
//     error: anyhow::Error,
// }

#[derive(Debug)]
pub enum C2Error {
    Unauthorized,
    Error(anyhow::Error),
}

impl From<anyhow::Error> for C2Error {
    fn from(error: anyhow::Error) -> Self {
        C2Error::Error(error)
    }
}

// impl Debug for C2Error {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "C2Error {{ error: {} }}", self.error)
//     }
// }

impl Display for C2Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            C2Error::Unauthorized => write!(f, "Unauthorized"),
            C2Error::Error(error) => write!(f, "Error: {}", error),
        }
    }
}

impl actix_web::error::ResponseError for C2Error {
    fn error_response(&self) -> actix_web::HttpResponse {
        // actix_web::HttpResponse::InternalServerError().json(self.error.to_string())
        match self {
            C2Error::Unauthorized => actix_web::HttpResponse::Unauthorized().finish(),
            C2Error::Error(_) => actix_web::HttpResponse::InternalServerError().finish(),
        }
    }
}
