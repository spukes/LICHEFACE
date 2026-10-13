use std::sync::mpsc::Sender;
use futures_util::StreamExt;
use crate::AppEvent;
use crate::lichess::types::*;

const BASE: &str = "https://lichess.org";

pub async fn stream_game(token: &str, game_id: &str, tx: Sender<AppEvent>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("LICHEFACE/0.1")
        .build()?;

    let resp = client
        .get(format!("{BASE}/api/board/game/stream/{game_id}"))
        .bearer_auth(token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let err_msg = format!("Lichess stream error ({status}): {body}");
        eprintln!("{err_msg}");
        let _ = tx.send(AppEvent::Error(err_msg));
        return Ok(());
    }

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let mut game_finished = false;

    while let Some(chunk) = stream.next().await {
        if game_finished {
            break;
        }
        let chunk = chunk?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete NDJSON lines
        while let Some(pos) = buf.find('\n') {
            let line: String = buf.drain(..=pos).collect();
            let line = line.trim();
            if line.is_empty() { continue; }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let event_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match event_type {
                    "gameFull" => {
                        let speed = val.get("speed").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let rated = val.get("rated").and_then(|v| v.as_bool()).unwrap_or(false);
                        let initial_time_secs = val.get("clock")
                            .and_then(|c| c.get("initial"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        let increment_secs = val.get("clock")
                            .and_then(|c| c.get("increment"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        let variant = val.get("variant")
                            .and_then(|v| v.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Standard")
                            .to_string();

                        let white_obj = val.get("white");
                        let white_is_ai = crate::lichess::types::is_ai_player(white_obj);
                        let white_ai_level = white_obj.and_then(|w| w.get("aiLevel").or_else(|| w.get("ai"))).and_then(|v| v.as_i64()).unwrap_or(1);
                        let white_name = if white_is_ai {
                            white_obj.and_then(|w| w.get("name").or_else(|| w.get("username")))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Stockfish Nivel {white_ai_level}"))
                        } else {
                            white_obj.and_then(|w| w.get("name").or_else(|| w.get("username")).or_else(|| w.get("id")))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Blancas")
                                .to_string()
                        };
                        let white_id = white_obj.and_then(|w| w.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string());
                        let white_rating = white_obj.and_then(|w| w.get("rating")).and_then(|v| v.as_u64()).map(|v| v as u32)
                            .or_else(|| if white_is_ai { Some(800 + (white_ai_level as u32) * 200) } else { None });
                        let white_title = white_obj.and_then(|w| w.get("title")).and_then(|v| v.as_str()).map(|s| s.to_string());

                        let black_obj = val.get("black");
                        let black_is_ai = crate::lichess::types::is_ai_player(black_obj);
                        let black_ai_level = black_obj.and_then(|b| b.get("aiLevel").or_else(|| b.get("ai"))).and_then(|v| v.as_i64()).unwrap_or(1);
                        let black_name = if black_is_ai {
                            black_obj.and_then(|b| b.get("name").or_else(|| b.get("username")))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Stockfish Nivel {black_ai_level}"))
                        } else {
                            black_obj.and_then(|b| b.get("name").or_else(|| b.get("username")).or_else(|| b.get("id")))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Negras")
                                .to_string()
                        };
                        let black_id = black_obj.and_then(|b| b.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string());
                        let black_rating = black_obj.and_then(|b| b.get("rating")).and_then(|v| v.as_u64()).map(|v| v as u32)
                            .or_else(|| if black_is_ai { Some(800 + (black_ai_level as u32) * 200) } else { None });
                        let black_title = black_obj.and_then(|b| b.get("title")).and_then(|v| v.as_str()).map(|s| s.to_string());

                        let details = GameDetails {
                            speed,
                            rated,
                            initial_time_secs,
                            increment_secs,
                            variant,
                            white_name,
                            white_id,
                            white_is_ai,
                            white_rating,
                            white_title,
                            black_name,
                            black_id,
                            black_is_ai,
                            black_rating,
                            black_title,
                        };
                        let _ = tx.send(AppEvent::GameDetailsLoaded(details));

                        // Parse initial state, injecting root status if missing
                        if let Some(state_val) = val.get("state") {
                            let mut state_val = state_val.clone();
                            if state_val.get("status").is_none() {
                                if let Some(root_status) = val.get("status") {
                                    if let Some(obj) = state_val.as_object_mut() {
                                        obj.insert("status".to_string(), root_status.clone());
                                    }
                                }
                            }
                            match serde_json::from_value::<GameState>(state_val) {
                                Ok(state) => {
                                    let _ = tx.send(AppEvent::GameState(state));
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse initial GameState: {e}");
                                }
                            }
                        }
                    }
                    "gameState" => {
                        match serde_json::from_value::<GameState>(val) {
                            Ok(state) => {
                                let is_over = state.status != "started" && state.status != "created";
                                let _ = tx.send(AppEvent::GameState(state));
                                if is_over {
                                    game_finished = true;
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse gameState: {e}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

