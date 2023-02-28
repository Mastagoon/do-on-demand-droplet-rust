use serde_json::Value;

const GET_ALL_DROPLETS: String = "https://api.digitalocean.com/v2/droplets".to_string();

struct ApiResponse {
    status: u32,
    error: Option<String>,
    data: Option<Value>,
}

pub async fn parse_json() {}

pub async fn get(url: String, _params: Option<Vec<String>>) -> ApiResponse {
    let response = reqwest::get(&url).await;
    if response.is_err() {
        return ApiResponse {
            status: 500,
            error: Some("Internal server error.".to_string()),
            data: None,
        };
    };
    let result = response.ok().unwrap();
    // get json from result
    let text = result.text().await;
    if text.is_err() {
        return ApiResponse {
            status: 500,
            error: Some("Failed to parse".to_string()),
            data: None,
        };
    };
    let json: Value = serde_json::from_str(&text.ok().unwrap()).unwrap();
    // ApiResponse{
    // status: result.status,
    // error: None,
    // }
    // let result = match reqwest::get(&url).await {
    // Ok(r) => match r.text().await {
    // Ok(t) => t,
    // Err(_) => "error".to_string(),
    // },
    // Err(_) => panic!(),
    // };

    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    return body;
}

pub async fn post(url: String, body: String) -> String {
    let client = reqwest::Client::new();
    let res = client.post(url).body(body).send().await.unwrap();
    let body = res.text().await.unwrap();
    return body;
}

// export const getAllDroplets = async () => {
// const result = await get(URLS.GET_ALL_DROPLETS)
// if (result.status !== 200)
// Log.error(`getAllDroplets: ${result.error ?? result.status}`)
// return result.body.droplets as {
// id: number
// name: string
// networks: { v4: { type: string; ip_address: string }[] }
// status: string
// }[]
//
// }

pub async fn getAllDroplets() {
    let result = get(GET_ALL_DROPLETS, None).await;
}
