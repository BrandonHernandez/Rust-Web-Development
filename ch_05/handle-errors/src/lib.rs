/// Error
/// 

use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden}, 
    http::StatusCode,
    reject::Reject,
    Rejection, 
    Reply,
};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            },
            Error::MissingParameters => {
                write!(f, "Missing parameter")
            },
            Error::QuestionNotFound => {
                write!(f, "Question not found")
            },
        }
    }
}

// Marker trait. This allows the Error to be returned in a Warp route handler.
impl Reject for Error {}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } 
    else {
        Ok(warp::reply::with_status(
            format!("{:?}", r),
            StatusCode::NOT_FOUND,
        ))
    }
}
