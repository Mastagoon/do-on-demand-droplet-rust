use serde::{Deserialize, Serialize};

use crate::{
    api::{self, CreateDropletBody},
    send,
};
use std::env::var;

pub async fn is_server_up() -> bool {
    let droplet_name = var("DROPLET_NAME").expect("DROPLET_NAME must be set");
    let result = api::get_all_droplets().await;
    if result.is_empty() {
        return false;
    }
    let index = result.iter().position(|d| d.name == droplet_name);
    if match index {
        Some(_) => true,
        None => false,
    } {
        return true;
    }
    true
}

pub async fn spawn_new_server() {
    let snapshot_name = var("SNAPSHOT_NAME").expect("SNAPSHOT_NAME must be set");
    let server_up = is_server_up().await;
    if server_up {
        send("Server is already up");
        return;
    }
    // find latest snapshot
    let snapshots = api::get_snapshot_list().await;
    if snapshots.is_empty() {
        return;
    }
    let index = snapshots.iter().position(|s| s.name == snapshot_name);
    let snapshot = match index {
        Some(i) => snapshots.get(i).unwrap(),
        None => {
            send("No snapshot found");
            return;
        }
    };
    // create new droplet
    let id = &snapshot.id;
    let droplet = create_droplet(id).await;
    if !droplet {
        send("Failed to create droplet");
        return;
    }
    send("Droplet created, awaiting IP...");
    while true {
        std::thread::sleep(std::time::Duration::from_secs(20));
        println!("Waiting for IP...");
        let d = match api::get_droplet_by_id(&droplet.id.to_string()).await {
            Some(d) => d,
            None => {
                send("Failed to get droplet");
                return;
            }
        };
        if d.networks.is_none() {
            continue;
        }
        let ip = match d
            .networks
            .unwrap()
            .v4
            .iter()
            .position(|n| n.r#type == "public")
        {
            Some(i) => d.networks.unwrap().v4.get(i).unwrap().ip_address,
            None => {
                continue;
            }
        };
        send(&format!("Droplet created, IP: {}", ip));
        break;
    }
}

pub async fn create_droplet(snapshot_id: &str) -> bool {
    let payload = CreateDropletBody {
        name: var("DROPLET_NAME").expect("DROPLET_NAME must be set"),
        region: var("DROPLET_REGION").expect("DROPLET_REGION must be set"),
        size: var("DROPLET_SIZE").expect("DROPLET_SIZE must be set"),
        image: snapshot_id.to_string(),
        ssh_keys: var("SSH_FINGERPRINT").expect("SSH_FINGERPRINT must be set"),
    };
    let result = api::create_droplet(payload).await;
    if result {
        send("Droplet created");
        true
    } else {
        send("Droplet creation failed");
        false
    }
}

pub async fn kill_server() {}
pub async fn update_snapshot() {}
