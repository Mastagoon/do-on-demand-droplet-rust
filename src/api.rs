use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::var;

#[derive(Serialize, Deserialize, Debug)]
struct Droplet {
    id: u32,
    name: String,
    memory: u32,
    vcpus: u32,
    disk: u32,
    locked: bool,
    status: String,
    kernel: Option<Value>,
    created_at: String,
    snapshot_ids: Vec<u32>,
    image: Option<Value>,
    volume_ids: Vec<u32>,
    size: Option<Value>,
    size_slug: String,
    networks: Option<Value>,
    region: Option<Value>,
    tags: Vec<String>,
    vpc_uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DropletCreate {
    name: String,
    region: String,
    size: String,
    image: String,
    ssh_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllDropletsResponse {
    droplets: Vec<Droplet>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateDropletResponse {
    droplet: Droplet,
}

const GET_ALL_DROPLETS_URL: &str = "https://api.digitalocean.com/v2/droplets";
const CREATE_DROPLET_URL: &str = "https://api.digitalocean.com/v2/droplets";
const DELETE_DROPLET_URL: &str = "https://api.digitalocean.com/v2/droplets/";

struct ApiResponse {
    status: u32,
    error: Option<String>,
    data: Option<Value>,
}

async fn get(url: &str, _params: Option<Vec<String>>) -> ApiResponse {
    let client = reqwest::Client::new();
    println!("GET -> {}", url);
    let response = match client
        .get(url)
        .header(
            "Authorization",
            ["Bearer ", &var("DO_TOKEN").expect("DO_TOKEN not set")].join(""),
        )
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return ApiResponse {
                status: 500,
                error: Some(e.to_string()),
                data: None,
            }
        }
    };

    println!("{} <-", response.status());

    ApiResponse {
        status: 200,
        error: None,
        data: response.json().await.ok(),
    }
}

async fn post(url: &str, body: String) -> ApiResponse {
    let client = reqwest::Client::new();
    println!("POST -> {}", url);
    let response = match client.post(url).body(body).send().await {
        Ok(r) => r,
        Err(e) => {
            return ApiResponse {
                status: 500,
                error: Some(e.to_string()),
                data: None,
            }
        }
    };
    println!("{} <-", response.status());
    return ApiResponse {
        status: 200,
        error: None,
        data: response.json().await.ok(),
    };
}

async fn delete(url: &str) -> ApiResponse {
    let client = reqwest::Client::new();
    println!("DELETE -> {}", url);
    let response = match client.delete(url).send().await {
        Ok(r) => r,
        Err(e) => {
            return ApiResponse {
                status: 500,
                error: Some(e.to_string()),
                data: None,
            }
        }
    };
    println!("{} <-", response.status());
    return ApiResponse {
        status: 200,
        error: None,
        data: response.json().await.ok(),
    };
}

pub async fn get_all_droplets() -> AllDropletsResponse {
    let result = get(GET_ALL_DROPLETS_URL, None).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return AllDropletsResponse { droplets: vec![] };
    }
    let data: AllDropletsResponse = serde_json::from_value(result.data.unwrap()).unwrap();
    println!("{:?}", data);
    data
}

pub async fn create_droplet() -> bool {
    let body = DropletCreate {
        name: var("DROPLET_NAME").expect("DROPLET_NAME not set"),
        region: var("DROPLET_REGION").expect("DROPLET_REGION not set"),
        size: var("DROPLET_SIZE").expect("DROPLET_SIZE not set"),
        image: var("DO_IMAGE_ID").expect("DROPLET_IMAGE not set"),
        ssh_keys: var("SSH_FINGERPRINT")
            .expect("SSH_FINGERPRINT not set")
            .split(",")
            .map(|s| s.to_string())
            .collect(),
    };
    let result = post(CREATE_DROPLET_URL, serde_json::to_string(&body).unwrap()).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return false;
    }
    println!("create droplet result: {:?}", result.data);
    let data: CreateDropletResponse = serde_json::from_value(result.data.unwrap()).unwrap();
    println!("{:?}", data);
    true
}
