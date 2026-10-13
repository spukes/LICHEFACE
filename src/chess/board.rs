use shakmaty::{Chess, Position, Square, Move, Role, Color, Piece, Board};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;

#[derive(Debug, Clone)]
pub struct BoardState {
    pub position: Chess,
    #[allow(dead_code)]
    pub flipped: bool,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            position: Chess::default(),
            flipped: false,
        }
    }
}

impl BoardState {
    #[allow(dead_code)]
    pub fn from_fen(fen: &str) -> Self {
        let pos: Chess = match fen.parse::<Fen>() {
            Ok(f) => f.into_position(shakmaty::CastlingMode::Standard).unwrap_or_default(),
            Err(_) => Chess::default(),
        };
        Self { position: pos, flipped: false }
    }

    pub fn from_moves(moves_str: &str) -> Self {
        let mut pos = Chess::default();
        for uci_str in moves_str.split_whitespace() {
            if let Ok(uci) = uci_str.parse::<UciMove>() {
                if let Ok(m) = uci.to_move(&pos) {
                    if let Ok(next) = pos.clone().play(&m) {
                        pos = next;
                    }
                }
            }
        }
        Self { position: pos, flipped: false }
    }

    pub fn board(&self) -> &Board {
        self.position.board()
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        self.position.legal_moves().into_iter().collect()
    }

    pub fn legal_destinations(&self, from: Square) -> Vec<Square> {
        self.legal_moves()
            .into_iter()
            .filter(|m| m.from() == Some(from))
            .map(|m| m.to())
            .collect()
    }

    pub fn make_uci_move(&mut self, uci: &str) -> bool {
        if let Ok(uci_move) = uci.parse::<UciMove>() {
            if let Ok(m) = uci_move.to_move(&self.position) {
                if let Ok(next) = self.position.clone().play(&m) {
                    self.position = next;
                    return true;
                }
            }
        }
        false
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board().piece_at(sq)
    }

    pub fn turn(&self) -> Color {
        self.position.turn()
    }

    pub fn is_check(&self) -> bool {
        self.position.is_check()
    }

    #[allow(dead_code)]
    pub fn is_game_over(&self) -> bool {
        self.position.is_game_over()
    }
}

#[allow(dead_code)]
pub fn piece_char(piece: Piece) -> &'static str {
    match (piece.role, piece.color) {
        (Role::King,   Color::White) => "♔",
        (Role::Queen,  Color::White) => "♕",
        (Role::Rook,   Color::White) => "♖",
        (Role::Bishop, Color::White) => "♗",
        (Role::Knight, Color::White) => "♘",
        (Role::Pawn,   Color::White) => "♙",
        (Role::King,   Color::Black) => "♚",
        (Role::Queen,  Color::Black) => "♛",
        (Role::Rook,   Color::Black) => "♜",
        (Role::Bishop, Color::Black) => "♝",
        (Role::Knight, Color::Black) => "♞",
        (Role::Pawn,   Color::Black) => "♟",
    }
}

/// Converts a Standard Algebraic Notation (SAN) string to Spanish piece initials:
/// K -> R (Rey), Q -> D (Dama), R -> T (Torre), B -> A (Alfil), N -> C (Caballo)
pub fn san_to_spanish(san: &str) -> String {
    let mut res = String::with_capacity(san.len());
    let mut chars = san.chars().peekable();
    if let Some(&first) = chars.peek() {
        match first {
            'K' => { res.push('R'); chars.next(); }
            'Q' => { res.push('D'); chars.next(); }
            'R' => { res.push('T'); chars.next(); }
            'B' => { res.push('A'); chars.next(); }
            'N' => { res.push('C'); chars.next(); }
            _ => {}
        }
    }
    while let Some(c) = chars.next() {
        if c == '=' {
            res.push('=');
            if let Some(promo) = chars.next() {
                match promo {
                    'Q' => res.push('D'),
                    'R' => res.push('T'),
                    'B' => res.push('A'),
                    'N' => res.push('C'),
                    other => res.push(other),
                }
            }
        } else {
            res.push(c);
        }
    }
    res
}

/// Converts a slice of UCI move strings into Spanish Algebraic Notation strings
pub fn uci_to_san_list(moves: &[String], initial_fen: Option<&str>) -> Vec<String> {
    use shakmaty::san::San;

    let mut pos: Chess = initial_fen
        .and_then(|f| f.parse::<Fen>().ok())
        .and_then(|f| f.into_position(shakmaty::CastlingMode::Standard).ok())
        .unwrap_or_default();

    let mut result = Vec::with_capacity(moves.len());
    for uci_str in moves {
        if let Ok(uci) = uci_str.parse::<UciMove>() {
            if let Ok(m) = uci.to_move(&pos) {
                let san = San::from_move(&pos, &m);
                let spanish_san = san_to_spanish(&san.to_string());
                result.push(spanish_san);
                if let Ok(next) = pos.clone().play(&m) {
                    pos = next;
                }
            } else {
                result.push(uci_str.clone());
            }
        } else {
            result.push(uci_str.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_san_generation_and_spanish_translation() {
        let uci_moves: Vec<String> = vec![
            "e2e4".into(), "e7e5".into(),
            "g1f3".into(), "b8c6".into(),
            "f1b5".into(), "a7a6".into(),
            "b5a4".into(), "g8f6".into(),
            "e1g1".into(), "f8e7".into(),
        ];
        let san_list = uci_to_san_list(&uci_moves, None);
        assert_eq!(
            san_list,
            vec!["e4", "e5", "Cf3", "Cc6", "Ab5", "a6", "Aa4", "Cf6", "O-O", "Ae7"]
        );
    }

    #[test]
    fn test_san_to_spanish_details() {
        assert_eq!(san_to_spanish("Nf3"), "Cf3");
        assert_eq!(san_to_spanish("Bb5"), "Ab5");
        assert_eq!(san_to_spanish("Rad1"), "Tad1");
        assert_eq!(san_to_spanish("N1f3"), "C1f3");
        assert_eq!(san_to_spanish("Qxd8#"), "Dxd8#");
        assert_eq!(san_to_spanish("Bxf7+"), "Axf7+");
        assert_eq!(san_to_spanish("O-O"), "O-O");
        assert_eq!(san_to_spanish("O-O-O"), "O-O-O");
        assert_eq!(san_to_spanish("e8=Q"), "e8=D");
        assert_eq!(san_to_spanish("exd8=N+"), "exd8=C+");
    }
}

