///

use warp::Filter;

#[tokio::main]
async fn main() {
    let s = "m+z-cycles";
    
    let hello = warp::get()
        .map(move || format!("{}", s));

    // let hello = warp::path("hello")
    //     .and(warp::path::param())
    //     .map(|name: String| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 1337))
        .await;    
}
