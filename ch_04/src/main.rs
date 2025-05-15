use warp::{
    Filter, 
    http::Method,
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::RwLock;

// #[derive(Clone, Serialize)]
#[derive(Debug, Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct AnswerId(String);

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    question_id: QuestionId,
}

async fn add_answer(
    store: Store, 
    params: HashMap<String, String>
) -> Result<impl warp::Reply, warp::Rejection> {
    let answer =  Answer {
        id: AnswerId(params.get("id").unwrap().to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(
            params.get("questionId").unwrap().to_string()
        ),
    };

    store.answers.write().await.insert(answer.id.clone(), answer);
    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}

// Route handler!
async fn get_questions(params: HashMap<String, String>, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    dbg!("{:#?}", &store);
    if !params.is_empty() {
        let mut pagination = extract_pagination(params)?;
        let res: Vec<Question> = store
            .questions
            .read()
            .await
            .values()
            .cloned()
            .collect();
                
        pagination = pagination.saturate(res.len());
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store
            .questions
            .read()
            .await
            .values()
            .cloned()
            .collect();
        Ok(warp::reply::json(&res))
    }

}

async fn get_one_question(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.read().await.get(&QuestionId(id)) {
        Some(q) => {
            return Ok(warp::reply::json(&q))
        },
        None => {
            return Err(warp::reject::custom(error::Error::QuestionNotFound))
        },
    }
    
}

async fn add_question(store: Store, question: Question) -> Result<impl warp::Reply, warp::Rejection> {
    store.questions.write().await
    .insert(
        question.id.clone(), 
        question,
    );

    // dbg!("{:#?}", store);

    Ok(warp::reply::with_status("Question added!", StatusCode::OK))
}

async fn update_question(id: String, store: Store, question: Question) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }

    // dbg!("{:#?}", store);

    Ok(warp::reply::with_status(
        "Question updated!",
        StatusCode::OK,
    ))
}

async fn delete_question(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_deleted_question) => {
            // dbg!(":#?", _deleted_question);
            return Ok(
                warp::reply::with_status(
                    "Question deleted",
                    StatusCode::OK,
                )
            )
        },
        None => {
            return Err(
                warp::reject::custom(error::Error::QuestionNotFound)
            )
        },
    }
}



mod error {
    use warp::{
        reject::Reject,
        filters::{body::BodyDeserializeError, cors::CorsForbidden}, 
        Rejection, 
        Reply,
        http::StatusCode,
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
}

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
    fn saturate(mut self, max_len: usize) -> Self {
        if self.end > max_len {
            println!("Saturating! Level: {}", max_len);
            self.end = max_len;
            println!("Start: {} End: {}", self.start, self.end);
        }
        self
    }
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, error::Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let mut pagination = Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(error::Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(error::Error::ParseError)?,
        };
        // println!("{:?}", pagination);
        pagination = pagination.sanitize();
        // println!("{:?}", pagination);
        return Ok(pagination);
    }
    Err(error::Error::MissingParameters)
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
        // .and(warp::path("another"))  // http://localhost:3030/questions/another
        .and(warp::path::end())         // marks the end of the path
        .and(warp::query())             // this gets the url parameters. Sets first param.
        .and(store_filter.clone())      // Is this a call to a closure? Did it capture the `store` variable? Sets second param.
        .and_then(get_questions);       // get_questions receives 2 params.    
    
    let get_one_question = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>()) // first param: Id
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_one_question);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())      // first param: Store
        .and(warp::body::json())        // second param: Question
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())     // first param: Id
        .and(warp::path::end())
        .and(store_filter.clone())              // second param: Store
        .and(warp::body::json())                // third param: Question
        .and_then(update_question);
    
    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())     // first param: id
        .and(warp::path::end())
        .and(store_filter.clone())              // second param: Store
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())              // first param: Store
        .and(warp::body::form())                // second param: Params (url-form-encoded)
        .and_then(add_answer);

    let routes = get_questions
        .or(get_one_question)
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .with(cors)
        .recover(error::return_error);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}