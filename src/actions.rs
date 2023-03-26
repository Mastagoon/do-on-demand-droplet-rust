#[allow(dead_code)]

const MAX_RETRIES: i32 = 5;

pub enum ActionResponse {
    SUCCESS(String),
    FAIL(String),
}

use serenity::{model::prelude::Message, prelude::Context};

use crate::api::{self, CreateDropletBody, Droplet};
use std::env::var;

async fn is_server_up() -> Option<Droplet> {
    let droplet_name = var("DROPLET_NAME").expect("DROPLET_NAME must be set");
    let result = api::get_all_droplets().await;
    if result.is_empty() {
        return None;
    }
    let droplet = result.into_iter().find(|d| d.name == droplet_name);
    droplet
}

pub async fn spawn_new_server(msg: &Message, ctx: &Context) {
    let snapshot_name = var("SNAPSHOT_NAME").expect("SNAPSHOT_NAME must be set");
    let active_droplet = is_server_up().await;
    if active_droplet.is_some() {
        if let Err(why) = msg.reply(&ctx.http, "Server already exists!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    }
    // find latest snapshot
    let snapshots = api::get_snapshot_list().await;
    if snapshots.is_empty() {
        if let Err(why) = msg.reply(&ctx.http, "No snapshots found!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    }
    let snapshot = snapshots.into_iter().find(|s| s.name == snapshot_name);
    if snapshot.is_none() {
        if let Err(why) = msg.reply(&ctx.http, "No snapshots found!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    }
    // create new droplet
    let id = snapshot.unwrap().id;
    let droplet_id = create_droplet(id).await;
    if droplet_id.is_none() {
        if let Err(why) = msg.reply(&ctx.http, "Failed to create droplet!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    }
    let droplet_id = droplet_id.unwrap();
    let mut err_count = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        println!("Waiting for IP...");
        let d = match api::get_droplet_by_id(droplet_id).await {
            Some(d) => d,
            None => {
                if err_count < MAX_RETRIES {
                    err_count += 1;
                    continue;
                } else {
                    if let Err(why) = msg
                        .reply(&ctx.http, "Failed to get droplet after max retries!")
                        .await
                    {
                        println!("Error sending message: {:?}", why);
                    };
                    return;
                }
            }
        };
        if d.networks.is_none() {
            continue;
        }
        let v4 = d.networks.unwrap().v4;
        let network = v4.into_iter().find(|n| n.r#type == "public");
        if network.is_none() {
            continue;
        }
        let ip = network.unwrap().ip_address;
        if let Err(why) = msg
            .reply(&ctx.http, format!("Server is up! IP: {}", ip))
            .await
        {
            println!("Error sending message: {:?}", why);
        };
        break;
    }
}

async fn create_droplet(snapshot_id: String) -> Option<u32> {
    let payload = CreateDropletBody {
        name: var("DROPLET_NAME").expect("DROPLET_NAME must be set"),
        region: var("DROPLET_REGION").expect("DROPLET_REGION must be set"),
        size: var("DROPLET_SIZE").expect("DROPLET_SIZE must be set"),
        image: snapshot_id.parse().unwrap_or(0),
        ssh_keys: var("SSH_FINGERPRINT")
            .expect("SSH_FINGERPRINT must be set")
            .split(",")
            .map(|s| s.to_string())
            .collect(),
    };
    let result = api::create_droplet(payload).await;
    match result {
        Some(d) => {
            println!("Droplet created: {}", d.id);
            Some(d.id)
        }
        None => {
            println!("Failed to create droplet");
            None
        }
    }
}

pub async fn kill_server(msg: &Message, ctx: &Context) {
    let droplet = is_server_up().await;
    if droplet.is_none() {
        if let Err(why) = msg.reply(&ctx.http, "Server does not exist!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    };
    let droplet = droplet.unwrap();
    // shutdown
    let shutdown_result = api::shutdown_droplet(droplet.id).await;
    if !shutdown_result {
        if let Err(why) = msg.reply(&ctx.http, "Failed to shutdown droplet!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    };
    // take snapshot
    let result = update_snapshot(droplet.id).await;
    if !result {
        if let Err(why) = msg
            .reply(&ctx.http, "Failed to take snapshot! Exiting.")
            .await
        {
            println!("Error sending message: {:?}", why);
        };
        return;
    };
    let result = api::delete_droplet(droplet.id.to_string()).await;
    if !result {
        if let Err(why) = msg.reply(&ctx.http, "Failed to delete droplet!").await {
            println!("Error sending message: {:?}", why);
        };
        return;
    };
    if let Err(why) = msg.reply(&ctx.http, "Server killed!").await {
        println!("Error sending message: {:?}", why);
    };
}

async fn update_snapshot(droplet_id: u32) -> bool {
    let list = api::get_snapshot_list().await;
    let name = var("SNAPSHOT_NAME").expect("SNAPSHOT_NAME must be set");
    let old_snapshot = list.iter().find(|s| s.name == name);

    let create_snapshot_result = api::create_snapshot(droplet_id).await;
    if !create_snapshot_result {
        println!("Snapshot failed.");
        return false;
    };
    loop {
        std::thread::sleep(std::time::Duration::from_secs(20));
        println!("creating snapshot...");
        let snapshots = api::get_snapshot_list().await;
        // #fixme for now this is the only way we can check if the snapshot is done.
        if snapshots.len() > list.len() {
            if old_snapshot.is_some() {
                let snapshot_id = &old_snapshot.unwrap().id;
                api::delete_snapshot(snapshot_id.to_owned()).await;
            }
            break true;
        }
    }
}
