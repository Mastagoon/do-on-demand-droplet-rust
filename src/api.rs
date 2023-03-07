use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env::var;

const DROPLETS_URL: &str = "https://api.digitalocean.com/v2/droplets";
const CREATE_DROPLET_URL: &str = "https://api.digitalocean.com/v2/droplets";
const DELETE_DROPLET_URL: &str = "https://api.digitalocean.com/v2/droplets/";
const SNAPSHOTS_URL: &str = "https://api.digitalocean.com/v2/snapshots";

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDropletBody {
    pub name: String,
    pub region: String,
    pub size: String,
    pub image: String,
    pub ssh_keys: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    created_at: String,
    regions: Vec<String>,
    resource_id: String,
    resource_type: String,
    min_disk_size: u32,
    size_gigabytes: f32,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AllSnapshotsResponse {
    snapshots: Vec<Snapshot>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropletNetworkV4 {
    pub ip_address: String,
    pub netmask: String,
    pub gateway: String,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropletNetwork {
    pub v4: Vec<DropletNetworkV4>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Droplet {
    pub id: u32,
    pub name: String,
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
    pub networks: Option<DropletNetwork>,
    region: Option<Value>,
    tags: Vec<String>,
    vpc_uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetDropletResponse {
    droplet: Droplet,
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

pub async fn get_all_droplets() -> Vec<Droplet> {
    let result = get(DROPLETS_URL, None).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return Vec::new();
    }
    let data: AllDropletsResponse = serde_json::from_value(result.data.unwrap()).unwrap();
    println!("{:?}", data);
    data.droplets
}

pub async fn create_droplet(body: CreateDropletBody) -> Option<Droplet> {
    let result = post(CREATE_DROPLET_URL, serde_json::to_string(&body).unwrap()).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return false;
    }
    println!("create droplet result: {:?}", result.data);
    let data: CreateDropletResponse = serde_json::from_value(result.data.unwrap()).unwrap();
    println!("{:?}", data);
    data.droplet
}

pub async fn drop_droplet(id: &str) -> bool {
    let result = delete(&[DELETE_DROPLET_URL, id].join("")).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return false;
    }
    println!("delete droplet result: {:?}", result.data);
    true
}

pub async fn get_snapshot_list() -> Vec<Snapshot> {
    let result = get(SNAPSHOTS_URL, None).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return Vec::new();
    }
    println!("get snapshot list result: {:?}", result.data);
    let data: AllSnapshotsResponse = serde_json::from_value(result.data.unwrap()).unwrap();

    data.snapshots
}

pub async fn delete_snapshot(id: &str) -> bool {
    let result = delete(&[SNAPSHOTS_URL, id].join("")).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return false;
    }
    println!("delete snapshot result: {:?}", result.data);
    true
}

pub async fn shutdown_droplet(id: &str) -> bool {
    let body = Action {
        r#type: "shutdown".to_string(),
    };
    let result = post(
        &[DROPLETS_URL, "/", id, "/actions"].join(""),
        serde_json::to_string(&body).unwrap(),
    )
    .await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return false;
    }
    println!("shutdown droplet result: {:?}", result.data);
    true
}

pub async fn get_droplet_by_id(id: &str) -> Option<Droplet> {
    let result = get(&[DROPLETS_URL, "/", id].join(""), None).await;
    if result.error.is_some() {
        println!("Error: {}", result.error.unwrap());
        return None;
    }
    println!("get droplet by id result: {:?}", result.data);
    let data: GetDropletResponse = serde_json::from_value(result.data.unwrap()).unwrap();
    Some(data.droplet)
}
