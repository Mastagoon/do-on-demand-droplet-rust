mod api;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let result = api::get_all_droplets().await;
    println!("{:?}", result);
}
