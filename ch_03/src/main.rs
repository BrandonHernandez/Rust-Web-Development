use std::str::FromStr;
use std::io::{Error, ErrorKind};

use warp::{
    Filter, 
    http::Method,
    filters::cors::CorsForbidden,
    reject::Reject, 
    Rejection, 
    Reply, 
    http::StatusCode
};

use serde::Serialize;

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct QuestionId(String);

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match !(id.is_empty()) {
            true => Ok(Self(String::from(id))),
            false => Err(
                Error::new(ErrorKind::InvalidInput, "No id provided")
            ),
        }
    }
}

impl Question {
    fn new(
        id: QuestionId, 
        title: String, 
        content: String, 
        tags: Option<Vec<String>>
    ) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{}, title: {}, content: {}, tags: {:?}", 
            self.id, self.title, self.content, self.tags
        )
    }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// This is our first route handler!
async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provided!"),
        String::from("First Question"),
        String::from("Content of Question"),
        Some(vec!(String::from("faq"))),
    );

    match question.id.0.parse::<i32>() {
        Ok(_) => {
            Ok(warp::reply::json(
                &question
            ))
        },
        Err(_) => {
            Err(warp::reject::custom(InvalidId))
        },
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "No valid ID presented".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found!".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(
            &[Method::PUT, Method::DELETE]
        );
    
    let get_items = warp::get()    
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    warp::serve(routes)
        .run(([192, 168, 30, 3], 3030))
        .await;
}