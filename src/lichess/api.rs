use reqwest::Client;
use crate::lichess::types::*;

const BASE: &str = "https://lichess.org";

fn client() -> Client {
    Client::builder()
        .user_agent("LICHEFACE/0.1")
        .build()
        .expect("Failed to build HTTP client")
}

pub async fn get_profile(token: &str) -> anyhow::Result<UserProfile> {
    let resp = client()
        .get(format!("{BASE}/api/account"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<UserProfile>()
        .await?;
    Ok(resp)
}

pub async fn create_seek(token: &str, time: u32, increment: u32, rated: bool) -> anyhow::Result<GameInfo> {
    // Check if time control meets Lichess Board API public seek criteria (Rapid or Classical)
    let total_est = time * 60 + 40 * increment;
    if time < 8 && total_est < 480 {
        return Err(anyhow::anyhow!(
            "Lichess Board API solo permite búsquedas públicas en ritmos Rápidos (10+0, 15+10, 20+0) o Clásicos (30+0).\nPara ritmos rápidos como Blitz o Bullet, juega contra Stockfish en la pestaña contigua."
        ));
    }

    let resp = client()
        .post(format!("{BASE}/api/board/seek"))
        .bearer_auth(token)
        .form(&[
            ("rated", if rated { "true" } else { "false" }),
            ("time", &time.to_string()),
            ("increment", &increment.to_string()),
            ("color", "random"),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        let err_msg = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&err_body) {
            if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
                err.to_string()
            } else if let Some(global) = val.get("global").and_then(|v| v.as_array()) {
                global.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")
            } else {
                err_body
            }
        } else {
            err_body
        };
        return Err(anyhow::anyhow!("Lichess ({status}): {err_msg}"));
    }

    // The seek streams NDJSON; we grab the gameStart event
    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let t = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if t == "gameStart" {
                    if let Some(game) = val.get("game") {
                        let id = game["gameId"].as_str().or_else(|| game["id"].as_str()).unwrap_or("").to_string();
                        let color = game["color"].as_str().unwrap_or("white").to_string();
                        let fen = game.get("fen")
                            .and_then(|v| v.as_str())
                            .unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                            .to_string();
                        return Ok(GameInfo {
                            id,
                            color,
                            fen,
                            opponent: None,
                            seconds_left: (time * 60) as f64,
                        });
                    }
                }
            }
        }
    }
    Err(anyhow::anyhow!("Búsqueda finalizada sin emparejar oponente. Puedes intentar de nuevo o iniciar una partida contra Stockfish."))
}

pub async fn challenge_ai(
    token: &str,
    level: u8,
    limit_secs: u32,
    increment_secs: u32,
    color: &str,
) -> anyhow::Result<GameInfo> {
    let mut form_data = vec![
        ("level", level.to_string()),
        ("color", color.to_string()),
    ];
    let limit_str = limit_secs.to_string();
    let inc_str = increment_secs.to_string();
    if limit_secs > 0 {
        form_data.push(("clock.limit", limit_str));
        form_data.push(("clock.increment", inc_str));
    }

    let resp = client()
        .post(format!("{BASE}/api/challenge/ai"))
        .bearer_auth(token)
        .form(&form_data)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Error al crear partida contra IA ({status}): {err_body}"));
    }

    let val: serde_json::Value = resp.json().await?;
    let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // Check which side is the AI using robust helper
    let is_white_ai = crate::lichess::types::is_ai_player(val.get("white"));
    let is_black_ai = crate::lichess::types::is_ai_player(val.get("black"));

    let player_color = if is_white_ai && !is_black_ai {
        "black".to_string()
    } else if is_black_ai && !is_white_ai {
        "white".to_string()
    } else if color == "black" {
        "black".to_string()
    } else {
        "white".to_string()
    };

    let default_opp_name = format!("Stockfish Nivel {}", level);
    let opp_name = if player_color == "white" {
        val.get("black")
            .and_then(|b| b.get("name").or_else(|| b.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or(&default_opp_name)
            .to_string()
    } else {
        val.get("white")
            .and_then(|w| w.get("name").or_else(|| w.get("username")))
            .and_then(|v| v.as_str())
            .unwrap_or(&default_opp_name)
            .to_string()
    };

    Ok(GameInfo {
        id,
        color: player_color,
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        opponent: Some(Opponent {
            username: opp_name,
            rating: Some(800 + (level as u32) * 200),
        }),
        seconds_left: limit_secs as f64,
    })
}

pub async fn get_now_playing(token: &str) -> anyhow::Result<Vec<NowPlayingGame>> {
    let resp = client()
        .get(format!("{BASE}/api/account/playing"))
        .bearer_auth(token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Error al consultar partidas ({status}): {err_body}"));
    }

    let data: NowPlayingResponse = resp.json().await?;
    Ok(data.now_playing)
}

pub async fn make_move(token: &str, game_id: &str, uci_move: &str) -> anyhow::Result<()> {
    client()
        .post(format!("{BASE}/api/board/game/{game_id}/move/{uci_move}"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[allow(dead_code)]
pub async fn abort_game(token: &str, game_id: &str) -> anyhow::Result<()> {
    client()
        .post(format!("{BASE}/api/board/game/{game_id}/abort"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn resign_game(token: &str, game_id: &str) -> anyhow::Result<()> {
    let resp = client()
        .post(format!("{BASE}/api/board/game/{game_id}/resign"))
        .bearer_auth(token)
        .send()
        .await?;

    // A 400 here means the game is already over — treat as success
    if resp.status().is_success() || resp.status() == 400 {
        return Ok(());
    }
    resp.error_for_status()?;
    Ok(())
}

pub async fn offer_draw(token: &str, game_id: &str, accept: bool) -> anyhow::Result<()> {
    let value = if accept { "yes" } else { "no" };
    client()
        .post(format!("{BASE}/api/board/game/{game_id}/draw/{value}"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[allow(dead_code)]
pub async fn send_chat(token: &str, game_id: &str, text: &str) -> anyhow::Result<()> {
    client()
        .post(format!("{BASE}/api/board/game/{game_id}/chat"))
        .bearer_auth(token)
        .form(&[("room", "player"), ("text", text)])
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
