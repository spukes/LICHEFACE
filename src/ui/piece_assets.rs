use egui::Image;
use shakmaty::{Color, Role};
use crate::config::PieceSet;

macro_rules! piece_img {
    ($set:literal, $file:literal) => {
        Image::from_bytes(
            concat!("bytes://", $set, "_", $file, ".svg"),
            include_bytes!(concat!("../../assets/pieces/", $set, "/", $file, ".svg")),
        )
    };
}

/// Returns the authentic vector SVG or raster image for the given piece and piece set,
/// or None if the set is procedural (RetroPixel8Bit, RetroAscii).
pub fn get_piece_image(set: &PieceSet, role: Role, color: Color) -> Option<Image<'static>> {
    let piece_str = match (color, role) {
        (Color::White, Role::King)   => "wK",
        (Color::White, Role::Queen)  => "wQ",
        (Color::White, Role::Rook)   => "wR",
        (Color::White, Role::Bishop) => "wB",
        (Color::White, Role::Knight) => "wN",
        (Color::White, Role::Pawn)   => "wP",
        (Color::Black, Role::King)   => "bK",
        (Color::Black, Role::Queen)  => "bQ",
        (Color::Black, Role::Rook)   => "bR",
        (Color::Black, Role::Bishop) => "bB",
        (Color::Black, Role::Knight) => "bN",
        (Color::Black, Role::Pawn)   => "bP",
    };

    match set {
        PieceSet::RetroMonochrome | PieceSet::RetroGreenPhosphor => match piece_str {
            "wK" => Some(piece_img!("retro_outline", "wK")),
            "wQ" => Some(piece_img!("retro_outline", "wQ")),
            "wR" => Some(piece_img!("retro_outline", "wR")),
            "wB" => Some(piece_img!("retro_outline", "wB")),
            "wN" => Some(piece_img!("retro_outline", "wN")),
            "wP" => Some(piece_img!("retro_outline", "wP")),
            "bK" => Some(piece_img!("retro_outline", "bK")),
            "bQ" => Some(piece_img!("retro_outline", "bQ")),
            "bR" => Some(piece_img!("retro_outline", "bR")),
            "bB" => Some(piece_img!("retro_outline", "bB")),
            "bN" => Some(piece_img!("retro_outline", "bN")),
            "bP" => Some(piece_img!("retro_outline", "bP")),
            _ => None,
        },
        PieceSet::RetroCga | PieceSet::Alpha => match piece_str {
            "wK" => Some(piece_img!("alpha", "wK")),
            "wQ" => Some(piece_img!("alpha", "wQ")),
            "wR" => Some(piece_img!("alpha", "wR")),
            "wB" => Some(piece_img!("alpha", "wB")),
            "wN" => Some(piece_img!("alpha", "wN")),
            "wP" => Some(piece_img!("alpha", "wP")),
            "bK" => Some(piece_img!("alpha", "bK")),
            "bQ" => Some(piece_img!("alpha", "bQ")),
            "bR" => Some(piece_img!("alpha", "bR")),
            "bB" => Some(piece_img!("alpha", "bB")),
            "bN" => Some(piece_img!("alpha", "bN")),
            "bP" => Some(piece_img!("alpha", "bP")),
            _ => None,
        },
        PieceSet::RetroVga | PieceSet::CBurnett => match piece_str {
            "wK" => Some(piece_img!("cburnett", "wK")),
            "wQ" => Some(piece_img!("cburnett", "wQ")),
            "wR" => Some(piece_img!("cburnett", "wR")),
            "wB" => Some(piece_img!("cburnett", "wB")),
            "wN" => Some(piece_img!("cburnett", "wN")),
            "wP" => Some(piece_img!("cburnett", "wP")),
            "bK" => Some(piece_img!("cburnett", "bK")),
            "bQ" => Some(piece_img!("cburnett", "bQ")),
            "bR" => Some(piece_img!("cburnett", "bR")),
            "bB" => Some(piece_img!("cburnett", "bB")),
            "bN" => Some(piece_img!("cburnett", "bN")),
            "bP" => Some(piece_img!("cburnett", "bP")),
            _ => None,
        },
        PieceSet::Merida => match piece_str {
            "wK" => Some(piece_img!("merida", "wK")),
            "wQ" => Some(piece_img!("merida", "wQ")),
            "wR" => Some(piece_img!("merida", "wR")),
            "wB" => Some(piece_img!("merida", "wB")),
            "wN" => Some(piece_img!("merida", "wN")),
            "wP" => Some(piece_img!("merida", "wP")),
            "bK" => Some(piece_img!("merida", "bK")),
            "bQ" => Some(piece_img!("merida", "bQ")),
            "bR" => Some(piece_img!("merida", "bR")),
            "bB" => Some(piece_img!("merida", "bB")),
            "bN" => Some(piece_img!("merida", "bN")),
            "bP" => Some(piece_img!("merida", "bP")),
            _ => None,
        },
        PieceSet::Cardinal => match piece_str {
            "wK" => Some(piece_img!("cardinal", "wK")),
            "wQ" => Some(piece_img!("cardinal", "wQ")),
            "wR" => Some(piece_img!("cardinal", "wR")),
            "wB" => Some(piece_img!("cardinal", "wB")),
            "wN" => Some(piece_img!("cardinal", "wN")),
            "wP" => Some(piece_img!("cardinal", "wP")),
            "bK" => Some(piece_img!("cardinal", "bK")),
            "bQ" => Some(piece_img!("cardinal", "bQ")),
            "bR" => Some(piece_img!("cardinal", "bR")),
            "bB" => Some(piece_img!("cardinal", "bB")),
            "bN" => Some(piece_img!("cardinal", "bN")),
            "bP" => Some(piece_img!("cardinal", "bP")),
            _ => None,
        },
        PieceSet::RetroPixel8Bit | PieceSet::RetroAscii => None,
    }
}
