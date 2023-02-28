mod api;
use dotenv::dotenv;
use serde_json::{self, Value};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let result: Value = serde_json::from_str(&(test().await).to_string()).unwrap();
    // let name = result.get("name").expect("Nope.");
    println!("{}, \n{}", result["name"], result["id"]);
}

async fn test() -> String {
    let result = api::get(
        "https://jsonplaceholder.typicode.com/comments/3".to_string(),
        None,
    )
    .await;
    // println!("result: {}", result);
    result
}
