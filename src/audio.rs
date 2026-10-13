use std::io::Cursor;
use std::sync::mpsc::{channel, Sender};
use rodio::OutputStream;

pub static OPPONENT_MOVE_SOUND: &[u8] = include_bytes!("../assets/sounds/movimiento1.mp3");
pub static PLAYER_MOVE_SOUND: &[u8] = include_bytes!("../assets/sounds/movimiento2.mp3");

#[derive(Debug, Clone, Copy)]
pub enum SoundAction {
    OpponentMove,
    PlayerMove,
}

#[derive(Clone)]
pub struct AudioManager {
    tx: Sender<SoundAction>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (tx, rx) = channel::<SoundAction>();

        std::thread::Builder::new()
            .name("licheface-audio".into())
            .spawn(move || {
                let mut stream_pair: Option<(rodio::OutputStream, rodio::OutputStreamHandle)> = None;

                while let Ok(action) = rx.recv() {
                    // Inicialización perezosa / reconexión si no estaba listo
                    if stream_pair.is_none() {
                        match OutputStream::try_default() {
                            Ok(pair) => stream_pair = Some(pair),
                            Err(e) => eprintln!("[Audio] No se pudo inicializar el dispositivo de salida de audio: {e}"),
                        }
                    }

                    if let Some((_, ref stream_handle)) = stream_pair {
                        let (sound_data, label) = match action {
                            SoundAction::OpponentMove => (OPPONENT_MOVE_SOUND, "movimiento1.mp3"),
                            SoundAction::PlayerMove => (PLAYER_MOVE_SOUND, "movimiento2.mp3"),
                        };

                        let cursor = Cursor::new(sound_data);
                        match rodio::Decoder::new(cursor) {
                            Ok(source) => {
                                match rodio::Sink::try_new(stream_handle) {
                                    Ok(sink) => {
                                        sink.set_volume(1.0);
                                        sink.append(source);
                                        sink.detach();
                                    }
                                    Err(e) => eprintln!("[Audio] Error al crear Sink de audio: {e}"),
                                }
                            }
                            Err(e) => eprintln!("[Audio] Error al decodificar {label}: {e}"),
                        }
                    }
                }
            })
            .expect("No se pudo iniciar el hilo de audio");

        Self { tx }
    }

    pub fn play_opponent_move(&self) {
        let _ = self.tx.send(SoundAction::OpponentMove);
    }

    pub fn play_player_move(&self) {
        let _ = self.tx.send(SoundAction::PlayerMove);
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::Decoder;

    #[test]
    fn test_mp3_decodes_successfully() {
        let cursor = Cursor::new(OPPONENT_MOVE_SOUND);
        let decoder = Decoder::new(cursor);
        assert!(decoder.is_ok(), "Failed to decode MP3 with rodio: {:?}", decoder.err());

        let cursor2 = Cursor::new(PLAYER_MOVE_SOUND);
        let decoder2 = Decoder::new(cursor2);
        assert!(decoder2.is_ok(), "Failed to decode PLAYER_MOVE_SOUND with rodio: {:?}", decoder2.err());
    }

    #[test]
    fn test_output_stream_and_play() {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            let cursor = Cursor::new(OPPONENT_MOVE_SOUND);
            let res = stream_handle.play_once(cursor);
            assert!(res.is_ok(), "play_once failed: {:?}", res.err());
            std::thread::sleep(std::time::Duration::from_millis(50));
        } else {
            eprintln!("No audio device in test environment, skipping play test");
        }
    }

    #[test]
    fn test_mp3_properties() {
        use rodio::Source;
        let cursor = Cursor::new(OPPONENT_MOVE_SOUND);
        let decoder = Decoder::new(cursor).unwrap();
        println!("Channels: {}", decoder.channels());
        println!("Sample rate: {}", decoder.sample_rate());
        println!("Total duration: {:?}", decoder.total_duration());
    }
}
