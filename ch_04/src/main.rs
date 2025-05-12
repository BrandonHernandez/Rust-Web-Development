use warp::{
    Filter, 
    http::Method,
    filters::cors::CorsForbidden,
    reject::Reject, 
    Rejection, 
    Reply, 
    http::StatusCode
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// #[derive(Clone, Serialize)]
#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json!")
    }
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

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

// Route handler!
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
   
    if !params.is_empty() {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        let pagination = extract_pagination(params)?;
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        // To get the whole Struct
        // let res = store;
        Ok(warp::reply::json(&res))
    }

}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
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
    } else {
        Ok(warp::reply::with_status(
            "Route not found!".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
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
        }
    }
}

// Marker trait. This allows the Error to be returned in a Warp route handler.
impl Reject for Error {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

// This one was my idea
impl Pagination {
    fn sanitize(mut self) -> Self {
        if self.start > self.end {
            let saved_start = self.start;
            self.start = self.end;
            self.end = saved_start;
        }
        self
    }
    fn verify_range(mut self, max_len: usize) -> () {
        
    }
}


fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let mut pagination = Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        };
        // println!("{:?}", pagination);
        pagination = pagination.sanitize();
        // println!("{:?}", pagination);
        return Ok(pagination);
    }
    Err(Error::MissingParameters)
}

#[tokio::main]
async fn main() {

    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(
            &[Method::PUT, Method::DELETE]
        );

    let get_questions = warp::get()    
        .and(warp::path("questions"))   // http://localhost:3030/questions
        // .and(warp::path("another"))   // http://localhost:3030/questions/another
        .and(warp::path::end())         // marks the end of the path
        .and(warp::query())             // this gets the url parameters. Sets first param.
        .and(store_filter)              // Is this a call to a closure? Did it capture the `store` variable? Sets second param.
        .and_then(get_questions)        // get_questions receives 2 params.    
        .recover(return_error);         // if get_questions fails, I think...

    let routes = get_questions.with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}