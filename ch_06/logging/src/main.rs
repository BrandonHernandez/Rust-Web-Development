#![warn(clippy::all)]

use handle_errors::return_error;
use warp::{Filter, http::Method};

mod routes;
mod store;
mod types;

// These uses shouldn't be required. Find out what's going on...
use crate::routes::answer::add_answer;
use crate::routes::question::add_question;
use crate::routes::question::delete_question;
use crate::routes::question::get_one_question;
use crate::routes::question::get_questions;
use crate::routes::question::update_question;
use crate::store::Store;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    log::error!("This is an error!");
    log::info!("This is info!");
    log::warn!("This is a warning!");

    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {} {:?} from {} with {:?}",
            info.method(),
            info.path(),
            info.status(),
            info.elapsed(),
            info.remote_addr().unwrap(),
            info.request_headers()
        );
    });

    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE]);

    let get_questions = warp::get()
        .and(warp::path("questions")) // http://localhost:3030/questions
        // .and(warp::path("another"))  // http://localhost:3030/questions/another
        .and(warp::path::end()) // marks the end of the path
        .and(warp::query()) // this gets the url parameters. Sets first param.
        .and(store_filter.clone()) // Is this a call to a closure? Did it capture the `store` variable? Sets second param.
        .and_then(get_questions); // get_questions receives 2 params.    

    let get_one_question = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>()) // first param: Id
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_one_question);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone()) // first param: Store
        .and(warp::body::json()) // second param: Question
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>()) // first param: Id
        .and(warp::path::end())
        .and(store_filter.clone()) // second param: Store
        .and(warp::body::json()) // third param: Question
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>()) // first param: id
        .and(warp::path::end())
        .and(store_filter.clone()) // second param: Store
        .and_then(delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone()) // first param: Store
        .and(warp::body::form()) // second param: Params (url-form-encoded)
        .and_then(add_answer);

    let routes = get_questions
        .or(get_one_question)
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .with(cors)
        .with(log)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

}
