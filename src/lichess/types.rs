use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub perfs: Perfs,
    pub online: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Perfs {
    #[serde(default)]
    pub bullet: PerfEntry,
    #[serde(default)]
    pub blitz: PerfEntry,
    #[serde(default)]
    pub rapid: PerfEntry,
    #[serde(default)]
    pub classical: PerfEntry,
    #[serde(default)]
    pub puzzle: PerfEntry,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerfEntry {
    #[serde(default)]
    pub rating: u32,
    #[serde(default)]
    pub games: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub color: String, // "white" | "black"
    pub fen: String,
    #[serde(default)]
    pub opponent: Option<Opponent>,
    #[serde(rename = "secondsLeft", default)]
    pub seconds_left: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opponent {
    pub username: String,
    pub rating: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub moves: String,
    #[serde(default, deserialize_with = "deserialize_time")]
    pub wtime: u64,
    #[serde(default, deserialize_with = "deserialize_time")]
    pub btime: u64,
    #[serde(default, deserialize_with = "deserialize_time")]
    pub winc: u64,
    #[serde(default, deserialize_with = "deserialize_time")]
    pub binc: u64,
    #[serde(default = "default_status", deserialize_with = "deserialize_status")]
    pub status: String,
    #[serde(default)]
    pub winner: Option<String>,
}

fn default_status() -> String {
    "started".to_string()
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(s)) = map.get("name") {
                Ok(s.clone())
            } else {
                Ok("started".to_string())
            }
        }
        _ => Ok("started".to_string()),
    }
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(u)
            } else if let Some(f) = n.as_f64() {
                Ok(f as u64)
            } else {
                Ok(0)
            }
        }
        _ => Ok(0),
    }
}

/// Robustly checks if a JSON player object represents a Stockfish AI player
pub fn is_ai_player(player_val: Option<&serde_json::Value>) -> bool {
    let Some(p) = player_val else { return false; };
    if let Some(ai) = p.get("ai") {
        if let Some(b) = ai.as_bool() {
            if b { return true; }
        }
        if let Some(n) = ai.as_i64() {
            if n > 0 { return true; }
        }
    }
    if let Some(level) = p.get("aiLevel") {
        if let Some(n) = level.as_i64() {
            if n > 0 { return true; }
        }
    }
    if let Some(id) = p.get("id").and_then(|v| v.as_str()) {
        if id.eq_ignore_ascii_case("stockfish") {
            return true;
        }
    }
    if let Some(name) = p.get("name").and_then(|v| v.as_str()) {
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("stockfish") {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameDetails {
    #[serde(default)]
    pub speed: String, // "bullet", "blitz", "rapid", "classical"
    #[serde(default)]
    pub rated: bool,
    #[serde(default)]
    pub initial_time_secs: u32,
    #[serde(default)]
    pub increment_secs: u32,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub white_name: String,
    #[serde(default)]
    pub white_id: Option<String>,
    #[serde(default)]
    pub white_is_ai: bool,
    #[serde(default)]
    pub white_rating: Option<u32>,
    #[serde(default)]
    pub white_title: Option<String>,
    #[serde(default)]
    pub black_name: String,
    #[serde(default)]
    pub black_id: Option<String>,
    #[serde(default)]
    pub black_is_ai: bool,
    #[serde(default)]
    pub black_rating: Option<u32>,
    #[serde(default)]
    pub black_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlayingResponse {
    #[serde(rename = "nowPlaying", default)]
    pub now_playing: Vec<NowPlayingGame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlayingGame {
    #[serde(rename = "fullId", default)]
    pub full_id: String,
    #[serde(rename = "gameId")]
    pub game_id: String,
    pub fen: String,
    pub color: String,
    #[serde(rename = "lastMove", default)]
    pub last_move: Option<String>,
    #[serde(rename = "isMyTurn", default)]
    pub is_my_turn: bool,
    #[serde(rename = "secondsLeft", default)]
    pub seconds_left: Option<f64>,
    pub opponent: Option<Opponent>,
}

// Stream event types
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GameFullEvent {
    pub id: String,
    pub white: GamePlayer,
    pub black: GamePlayer,
    pub state: GameState,
    #[serde(rename = "initialFen")]
    pub initial_fen: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GamePlayer {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub rating: Option<u32>,
    #[serde(default)]
    pub id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_deserialization_standard() {
        let json = r#"{
            "type": "gameState",
            "moves": "e2e4 e7e5",
            "wtime": 300000,
            "btime": 298000,
            "winc": 0,
            "binc": 0,
            "status": "started"
        }"#;
        let state: GameState = serde_json::from_str(json).expect("Standard gameState should parse");
        assert_eq!(state.moves, "e2e4 e7e5");
        assert_eq!(state.wtime, 300000);
        assert_eq!(state.btime, 298000);
        assert_eq!(state.status, "started");
    }

    #[test]
    fn test_game_state_deserialization_missing_status_and_time() {
        // As seen in gameFull state object or untimed games
        let json = r#"{
            "moves": "",
            "winc": 0,
            "binc": 0
        }"#;
        let state: GameState = serde_json::from_str(json).expect("Partial gameState should parse");
        assert_eq!(state.moves, "");
        assert_eq!(state.wtime, 0);
        assert_eq!(state.btime, 0);
        assert_eq!(state.status, "started");
    }

    #[test]
    fn test_game_state_deserialization_object_status() {
        // When status is given as an object { "name": "started" }
        let json = r#"{
            "moves": "d2d4",
            "wtime": 180000,
            "btime": 180000,
            "status": { "id": 20, "name": "started" }
        }"#;
        let state: GameState = serde_json::from_str(json).expect("Object status should parse");
        assert_eq!(state.moves, "d2d4");
        assert_eq!(state.status, "started");
    }

    #[test]
    fn test_game_state_deserialization_float_or_null_times() {
        let json = r#"{
            "moves": "e2e4",
            "wtime": 300000.0,
            "btime": null,
            "status": "mate",
            "winner": "white"
        }"#;
        let state: GameState = serde_json::from_str(json).expect("Float and null times should parse");
        assert_eq!(state.wtime, 300000);
        assert_eq!(state.btime, 0);
        assert_eq!(state.status, "mate");
        assert_eq!(state.winner.as_deref(), Some("white"));
    }

    #[test]
    fn test_is_ai_player() {
        // Normal human player
        let human = serde_json::json!({
            "id": "magnuscarlsen",
            "name": "Magnus Carlsen",
            "rating": 2850
        });
        assert!(!is_ai_player(Some(&human)));

        // Human player with ai: false
        let human_with_false = serde_json::json!({
            "id": "soyce",
            "name": "Soyce",
            "ai": false
        });
        assert!(!is_ai_player(Some(&human_with_false)));

        // Stockfish with level number
        let sf_level = serde_json::json!({
            "id": "stockfish",
            "name": "Stockfish level 3",
            "ai": 3
        });
        assert!(is_ai_player(Some(&sf_level)));

        // Stockfish with boolean true
        let sf_bool = serde_json::json!({
            "ai": true
        });
        assert!(is_ai_player(Some(&sf_bool)));

        // Stockfish by id
        let sf_id = serde_json::json!({
            "id": "Stockfish",
            "name": "Computer"
        });
        assert!(is_ai_player(Some(&sf_id)));

        // Stockfish by name prefix
        let sf_name = serde_json::json!({
            "name": "Stockfish AI Level 1"
        });
        assert!(is_ai_player(Some(&sf_name)));

        // Stockfish with aiLevel (as returned in Lichess gameFull stream)
        let sf_ai_level = serde_json::json!({
            "aiLevel": 1
        });
        assert!(is_ai_player(Some(&sf_ai_level)));

        let sf_ai_level_8 = serde_json::json!({
            "aiLevel": 8
        });
        assert!(is_ai_player(Some(&sf_ai_level_8)));

        // Empty / None
        assert!(!is_ai_player(None));
        assert!(!is_ai_player(Some(&serde_json::json!({}))));
    }
}

