use std::collections::HashMap;
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};
use handle_errors::Error;

// Route handler!
pub async fn get_questions(
    params: HashMap<String, String>, 
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {    
    
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

pub async fn get_one_question(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.read().await.get(&QuestionId(id)) {
        Some(q) => {
            return Ok(warp::reply::json(&q))
        },
        None => {
            return Err(warp::reject::custom(Error::QuestionNotFound))
        },
    }
    
}

pub async fn add_question(store: Store, question: Question) -> Result<impl warp::Reply, warp::Rejection> {
    store.questions.write().await
    .insert(
        question.id.clone(), 
        question,
    );

    // dbg!("{:#?}", store);

    Ok(warp::reply::with_status("Question added!", StatusCode::OK))
}

pub async fn update_question(id: String, store: Store, question: Question) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }

    // dbg!("{:#?}", store);

    Ok(warp::reply::with_status(
        "Question updated!",
        StatusCode::OK,
    ))
}

pub async fn delete_question(id: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
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
                warp::reject::custom(Error::QuestionNotFound)
            )
        },
    }
}