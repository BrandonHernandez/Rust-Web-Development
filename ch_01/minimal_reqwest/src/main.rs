///
/// 

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("http://localhost:1337")
        .await?;
        // .json::<HashMap<String, String>>()
        // .await?;
    println!("{:#?}", resp);
    Ok(())
}
