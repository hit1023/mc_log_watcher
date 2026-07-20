use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf; // Path を削除し PathBuf だけに（警告対策）
use std::sync::mpsc::channel;

// --- 修正ポイント1: 明示的にインポートを追加 ---
use serde_json; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let log_dirs_env = env::var("LOG_DIRS").unwrap_or_default();
    let log_dirs: Vec<PathBuf> = log_dirs_env.split(',').map(PathBuf::from).collect();
    let chatwork_token = env::var("CHATWORK_API_TOKEN").unwrap_or_default();
    let chatwork_room = env::var("CHATWORK_ROOM_ID").unwrap_or_default();
    let fastapi_url = "https://mc.s-quad.com/api/location";

    let client = Client::new();

    let re_command = Regex::new(r"<(?P<p>.+?)>\s+/?addloc\s+(?P<x>-?\d+\.?\d*)\s+(?P<y>-?\d+\.?\d*)\s+(?P<z>-?\d+\.?\d*)\s+(?P<w>\w+)\s+(?P<d>.+)")?;
    let re_login = Regex::new(r"\[Server thread/INFO\]: (\w+) joined the game")?;
    let re_logout = Regex::new(r"\[Server thread/INFO\]: (\w+) lost connection: Disconnected")?;

    let mut positions: HashMap<PathBuf, u64> = HashMap::new();
    
    for dir in &log_dirs {
        let path = dir.join("latest.log");
        if let Ok(metadata) = std::fs::metadata(&path) {
            positions.insert(path, metadata.len());
        }
    }

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    for dir in &log_dirs {
        if dir.exists() {
            watcher.watch(dir, RecursiveMode::NonRecursive)?;
            println!("🟢 Monitoring: {:?}", dir);
        }
    }

    loop {
        if let Ok(Ok(event)) = rx.recv() {
            let should_process = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_));
            if should_process {
                for path in event.paths {
                    if path.ends_with("latest.log") {
                        let server_name = if path.to_string_lossy().contains("minecraft_server2") { "Minecraft 2" } else { "Minecraft" };
                        
                        let mut current_pos = *positions.get(&path).unwrap_or(&0);

                        // Check actual file size to detect rotation
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let file_size = metadata.len();
                            // If file is smaller than current position, it must have been rotated/truncated.
                            // Or if it's a Create event, we should probably start from 0 strictly, but the size check covers most cases.
                            // However, explicit Create might mean completely new inode, so resetting is safe.
                            if matches!(event.kind, EventKind::Create(_)) || file_size < current_pos {
                                println!("🔄 Log rotation detected for {:?}. Resetting position.", path);
                                current_pos = 0;
                                positions.insert(path.clone(), 0);
                            }
                        }

                        if let Ok(file) = File::open(&path) {
                            let mut reader = BufReader::new(file);
                            
                            // If we failed to seek (e.g. file is smaller than pos causing error, though checking size above helps), 
                            // we fall back to 0. But we checked size above.
                            if reader.seek(SeekFrom::Start(current_pos)).is_err() {
                                // Fallback reset
                                let _ = reader.seek(SeekFrom::Start(0));
                                // current_pos = 0; // Removed unused assignment
                            }
                            
                            let lines = (&mut reader).lines(); 
                            for line in lines {
                                if let Ok(l) = line {
                                    handle_line(&l, server_name, &client, &re_command, &re_login, &re_logout, &chatwork_token, &chatwork_room, fastapi_url).await;
                                }
                            }
                            
                            if let Ok(new_pos) = reader.stream_position() {
                                positions.insert(path, new_pos);
                            }
                        }
                    }
                }
            }
        }
    }
}


async fn handle_line(line: &str, server: &str, client: &Client, re_cmd: &Regex, re_in: &Regex, re_out: &Regex, cw_token: &str, cw_room: &str, api_url: &str) {
    println!("Processing line: {}", line);
    // 1. addloc 処理
    if let Some(caps) = re_cmd.captures(line) {
        let data = serde_json::json!({
            "x": caps["x"].parse::<f64>().unwrap_or(0.0),
            "y": caps["y"].parse::<f64>().unwrap_or(0.0),
            "z": caps["z"].parse::<f64>().unwrap_or(0.0),
            "world": &caps["w"],
            "description": &caps["d"],
            "player": &caps["p"]
        });
        
        let _ = client.post(api_url).json(&data).send().await;
        let msg = format!("[{}] ✅ New location added by {}\n📍 Coordinates: X={}, Y={}, Z={} ({})\n📝 Description: {}", 
            server, &caps["p"], &caps["x"], &caps["y"], &caps["z"], &caps["w"], &caps["d"]);
        send_chatwork(client, cw_token, cw_room, &msg).await;
    }
    // 2. ログイン/アウト処理
    else if let Some(caps) = re_in.captures(line) {
        send_chatwork(client, cw_token, cw_room, &format!("[{}] 👤 Player `{}` has joined.", server, &caps[1])).await;
    } else if let Some(caps) = re_out.captures(line) {
        send_chatwork(client, cw_token, cw_room, &format!("[{}] 🚪 Player `{}` has left.", server, &caps[1])).await;
    }
}

async fn send_chatwork(client: &Client, token: &str, room: &str, body: &str) {
    let url = format!("https://api.chatwork.com/v2/rooms/{}/messages", room);
    let _ = client.post(url).header("X-ChatWorkToken", token).form(&[("body", body)]).send().await;
}
