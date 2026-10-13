use egui::{self, Color32, Pos2, Rect, Sense, Vec2};
use shakmaty::{Color as SColor, File, Rank, Role, Square};
use crate::theme::Theme;
use crate::config::PieceSet;

pub struct BoardWidget<'a> {
    pub board_state: &'a crate::chess::BoardState,
    pub selected: &'a mut Option<Square>,
    pub legal_dests: Vec<Square>,
    pub flipped: bool,
    pub theme: &'a Theme,
    pub piece_set: &'a PieceSet,
    pub show_coords: bool,
    #[allow(dead_code)]
    pub last_move: Option<(Square, Square)>,
    pub size: f32,
    pub retro_flavor: crate::config::RetroFlavor,
    pub interactive: bool,
    pub player_color: Option<SColor>,
}

impl<'a> BoardWidget<'a> {
    pub fn new(
        board_state: &'a crate::chess::BoardState,
        selected: &'a mut Option<Square>,
        theme: &'a Theme,
        piece_set: &'a PieceSet,
        show_coords: bool,
        last_move: Option<(Square, Square)>,
        size: f32,
    ) -> Self {
        let legal_dests = selected.map(|s| board_state.legal_destinations(s)).unwrap_or_default();
        Self {
            board_state,
            selected,
            theme,
            piece_set,
            show_coords,
            last_move,
            size,
            flipped: false,
            legal_dests,
            retro_flavor: crate::config::RetroFlavor::Vga,
            interactive: true,
            player_color: None,
        }
    }

    pub fn with_flavor(mut self, retro_flavor: crate::config::RetroFlavor) -> Self {
        self.retro_flavor = retro_flavor;
        self
    }

    pub fn with_interaction(mut self, interactive: bool, player_color: Option<SColor>) -> Self {
        self.interactive = interactive;
        self.player_color = player_color;
        self
    }

    fn format_uci(&self, from: Square, to: Square) -> String {
        let is_pawn = self.board_state.piece_at(from).map(|p| p.role == Role::Pawn).unwrap_or(false);
        if is_pawn && (to.rank() == Rank::Eighth || to.rank() == Rank::First) {
            format!("{}{}q", from, to)
        } else {
            format!("{}{}", from, to)
        }
    }

    /// Returns the UCI move string if a move occurred (via click or drag-and-drop)
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<String> {
        let sq_size = self.size / 8.0;
        let coord_margin = if self.show_coords { 20.0 } else { 0.0 };
        let board_size = Vec2::new(self.size + coord_margin, self.size + coord_margin);

        // Sense both click and drag for seamless dual interaction
        let (resp, painter) = ui.allocate_painter(board_size, if self.interactive { Sense::click_and_drag() } else { Sense::hover() });
        let board_origin = resp.rect.min + Vec2::new(coord_margin, 0.0);

        let mut move_result: Option<String> = None;

        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
        let is_dragging = self.interactive && resp.dragged();
        let drag_stopped = self.interactive && resp.drag_stopped();

        // Helper to check if a square contains a piece belonging to the human player
        let is_my_piece = |board_state: &crate::chess::BoardState, sq: Square, player_col: Option<SColor>| {
            if let Some(piece) = board_state.piece_at(sq) {
                if let Some(col) = player_col {
                    piece.color == col
                } else {
                    true
                }
            } else {
                false
            }
        };

        // 1. Drag start handling
        if self.interactive && resp.drag_started() {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(pos, board_origin, sq_size, self.flipped) {
                    if is_my_piece(self.board_state, sq, self.player_color) {
                        *self.selected = Some(sq);
                        self.legal_dests = self.board_state.legal_destinations(sq);
                    }
                }
            }
        }

        // 2. Drag release handling (Drop)
        if self.interactive && drag_stopped {
            if let Some(pos) = pointer_pos {
                if let Some(target_sq) = pos_to_square(pos, board_origin, sq_size, self.flipped) {
                    if let Some(from_sq) = *self.selected {
                        if target_sq != from_sq && self.legal_dests.contains(&target_sq) {
                            move_result = Some(self.format_uci(from_sq, target_sq));
                            *self.selected = None;
                        }
                    }
                }
            }
        }

        // 3. Regular Click handling (Click piece, then click destination)
        if self.interactive && resp.clicked() && move_result.is_none() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if let Some(sq) = pos_to_square(pos, board_origin, sq_size, self.flipped) {
                    if let Some(from) = *self.selected {
                        if self.legal_dests.contains(&sq) {
                            move_result = Some(self.format_uci(from, sq));
                            *self.selected = None;
                        } else {
                            *self.selected = if is_my_piece(self.board_state, sq, self.player_color) { Some(sq) } else { None };
                            self.legal_dests = self.selected.map(|s| self.board_state.legal_destinations(s)).unwrap_or_default();
                        }
                    } else {
                        *self.selected = if is_my_piece(self.board_state, sq, self.player_color) { Some(sq) } else { None };
                        self.legal_dests = self.selected.map(|s| self.board_state.legal_destinations(s)).unwrap_or_default();
                    }
                }
            }
        }

        let dragged_square = if is_dragging { *self.selected } else { None };

        // 4. Render Board Squares + Overlays
        for rank_idx in 0..8u8 {
            for file_idx in 0..8u8 {
                let (display_file, display_rank) = if self.flipped {
                    (7 - file_idx, rank_idx)
                } else {
                    (file_idx, 7 - rank_idx)
                };

                let sq = Square::from_coords(
                    File::new(display_file as u32),
                    Rank::new(display_rank as u32),
                );

                let x = board_origin.x + file_idx as f32 * sq_size;
                let y = board_origin.y + rank_idx as f32 * sq_size;
                let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(sq_size));

                // Base square color (clean retro look, no square color variation on last move)
                let is_light = (display_file + display_rank) % 2 == 1;
                let base_color = if is_light {
                    self.theme.retro_light_sq(self.retro_flavor)
                } else {
                    self.theme.retro_dark_sq(self.retro_flavor)
                };
                painter.rect_filled(rect, 0.0, base_color);

                // Selected highlight
                if *self.selected == Some(sq) {
                    let sel_color = self.theme.retro_selected_sq(self.retro_flavor, is_light);
                    painter.rect_filled(rect, 0.0, sel_color);
                    let sel_border = self.theme.accent_c(self.retro_flavor);
                    painter.rect_stroke(rect.shrink(1.0), 0.0, egui::Stroke::new(2.0_f32, sel_border));
                }

                // Hover target highlight while dragging
                if is_dragging {
                    if let Some(pos) = pointer_pos {
                        if let Some(hover_sq) = pos_to_square(pos, board_origin, sq_size, self.flipped) {
                            if hover_sq == sq && self.legal_dests.contains(&sq) {
                                // Sin mezclas alfa: la casilla mantiene su color y se enmarca con trazo retro nítido
                                let hover_border = self.theme.accent_c(self.retro_flavor);
                                painter.rect_stroke(
                                    rect.shrink(1.5),
                                    0.0,
                                    egui::Stroke::new(2.5_f32, hover_border),
                                );
                            }
                        }
                    }
                }

                // Legal destination dots
                if self.legal_dests.contains(&sq) {
                    let dot_color = self.theme.retro_legal_dot(self.retro_flavor, is_light);
                    if self.board_state.piece_at(sq).is_some() {
                        // Capture: ring
                        painter.circle_stroke(
                            rect.center(),
                            sq_size * 0.44,
                            egui::Stroke::new(sq_size * 0.08, dot_color),
                        );
                    } else {
                        // Empty: filled dot
                        painter.circle_filled(rect.center(), sq_size * 0.18, dot_color);
                    }
                }

                // Check highlight
                if self.board_state.is_check() {
                    let board = self.board_state.board();
                    let turn = self.board_state.turn();
                    if let Some(king_sq) = board.king_of(turn) {
                        if sq == king_sq {
                            let check_color = self.theme.retro_check_sq(self.retro_flavor, is_light);
                            painter.rect_filled(rect, 0.0, check_color);
                            let check_border = match self.retro_flavor {
                                crate::config::RetroFlavor::Vga => Color32::from_rgb(0, 0, 150),
                                crate::config::RetroFlavor::Cga => Color32::from_rgb(170, 0, 170),
                                crate::config::RetroFlavor::Monochrome => if is_light { Color32::WHITE } else { Color32::BLACK },
                                crate::config::RetroFlavor::GreenPhosphor => Color32::from_rgb(0, 25, 0),
                            };
                            painter.rect_stroke(rect.shrink(1.0), 0.0, egui::Stroke::new(2.5_f32, check_border));
                        }
                    }
                }

                // Static piece rendering on square (dimmed if currently being dragged)
                if let Some(piece) = self.board_state.piece_at(sq) {
                    let is_being_dragged = dragged_square == Some(sq);
                    let opacity = if is_being_dragged { 60 } else { 255 };

                    render_piece(
                        ui,
                        &painter,
                        rect.center(),
                        piece,
                        sq_size,
                        self.piece_set,
                        opacity,
                        !is_light,
                    );
                }

                // Coordinates
                if self.show_coords {
                    if file_idx == 0 {
                        let rank_label = format!("{}", display_rank + 1);
                        painter.text(
                            Pos2::new(board_origin.x - 12.0, y + sq_size / 2.0),
                            egui::Align2::RIGHT_CENTER,
                            rank_label,
                            egui::FontId::monospace(11.0),
                            self.theme.text2(),
                        );
                    }
                    if rank_idx == 7 {
                        let file_label = ["a","b","c","d","e","f","g","h"][display_file as usize];
                        painter.text(
                            Pos2::new(x + sq_size / 2.0, y + sq_size + 2.0),
                            egui::Align2::CENTER_TOP,
                            file_label,
                            egui::FontId::monospace(11.0),
                            self.theme.text2(),
                        );
                    }
                }
            }
        }

        // 5. Floating dragged piece following pointer with slight elevation
        if is_dragging {
            if let Some(from_sq) = dragged_square {
                if let Some(piece) = self.board_state.piece_at(from_sq) {
                    if let Some(pos) = pointer_pos {
                        // Subtle floating drop shadow
                        painter.circle_filled(
                            pos + Vec2::new(3.0, 5.0),
                            sq_size * 0.35,
                            Color32::from_black_alpha(70),
                        );

                        // Floating elevated piece (slightly larger scale)
                        render_piece(
                            ui,
                            &painter,
                            pos,
                            piece,
                            sq_size * 1.12,
                            self.piece_set,
                            255,
                            true,
                        );
                    }
                }
            }
        }

        move_result
    }
}

/// Renders a chess piece using authentic SVG vector graphics or retro procedural renderer
fn render_piece(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    center: Pos2,
    piece: shakmaty::Piece,
    sq_size: f32,
    piece_set: &PieceSet,
    alpha: u8,
    _is_dark_sq: bool,
) {
    let is_white = piece.color == SColor::White;

    if let Some(mut img) = crate::ui::piece_assets::get_piece_image(piece_set, piece.role, piece.color) {
        let piece_rect = Rect::from_center_size(center, Vec2::splat(sq_size * 0.90));
        
        // Authentic flavor tinting for retro piece sets
        match piece_set {
            PieceSet::RetroCga => {
                let tint_color = if is_white {
                    Color32::from_rgba_premultiplied(85, 255, 255, alpha) // Electric Cyan
                } else {
                    Color32::from_rgba_premultiplied(255, 85, 255, alpha) // Electric Magenta
                };
                img = img.tint(tint_color);
            }
            PieceSet::RetroGreenPhosphor => {
                let tint_color = if is_white {
                    Color32::from_rgba_premultiplied(102, 255, 102, alpha) // Glowing Green
                } else {
                    Color32::from_rgba_premultiplied(51, 255, 51, alpha) // Phosphor Green outline & details (100% green palette, zero white)
                };
                img = img.tint(tint_color);
            }
            PieceSet::RetroMonochrome => {
                let tint_color = if is_white {
                    Color32::from_rgba_premultiplied(255, 255, 255, alpha) // Pure White
                } else {
                    Color32::from_rgba_premultiplied(255, 255, 255, alpha) // Pure White vector contour & stark black
                };
                img = img.tint(tint_color);
            }
            _ => {
                if alpha < 255 {
                    img = img.tint(Color32::from_white_alpha(alpha));
                }
            }
        }
        img.paint_at(ui, piece_rect);
    } else {
        // Procedural Retro Piece sets: RetroPixel8Bit and RetroAscii
        let badge_size = sq_size * 0.72;
        let badge_rect = Rect::from_center_size(center, Vec2::splat(badge_size));

        let (bg_col, text_col, border_col) = match piece_set {
            PieceSet::RetroAscii => {
                if is_white {
                    (
                        Color32::from_rgba_premultiplied(0, 0, 160, alpha),
                        Color32::from_rgba_premultiplied(255, 255, 255, alpha),
                        Color32::from_rgba_premultiplied(255, 255, 85, alpha),
                    )
                } else {
                    (
                        Color32::from_rgba_premultiplied(0, 0, 40, alpha),
                        Color32::from_rgba_premultiplied(85, 255, 255, alpha),
                        Color32::from_rgba_premultiplied(180, 180, 180, alpha),
                    )
                }
            }
            _ => {
                // RetroPixel8Bit (Arcade High Contrast)
                if is_white {
                    (
                        Color32::from_rgba_premultiplied(240, 240, 240, alpha),
                        Color32::from_rgba_premultiplied(10, 10, 10, alpha),
                        Color32::from_rgba_premultiplied(85, 255, 255, alpha),
                    )
                } else {
                    (
                        Color32::from_rgba_premultiplied(20, 20, 20, alpha),
                        Color32::from_rgba_premultiplied(255, 255, 85, alpha),
                        Color32::from_rgba_premultiplied(255, 85, 255, alpha),
                    )
                }
            }
        };

        // Strict zero rounding for authentic retro hardware feel
        painter.rect_filled(badge_rect, 0.0, bg_col);
        painter.rect_stroke(badge_rect, 0.0, egui::Stroke::new(1.8_f32, border_col));

        let glyph = match (piece_set, piece.role, is_white) {
            (PieceSet::RetroAscii, Role::King, true) => "[K]",
            (PieceSet::RetroAscii, Role::Queen, true) => "[Q]",
            (PieceSet::RetroAscii, Role::Rook, true) => "[R]",
            (PieceSet::RetroAscii, Role::Bishop, true) => "[B]",
            (PieceSet::RetroAscii, Role::Knight, true) => "[N]",
            (PieceSet::RetroAscii, Role::Pawn, true) => "[P]",
            (PieceSet::RetroAscii, Role::King, false) => "[k]",
            (PieceSet::RetroAscii, Role::Queen, false) => "[q]",
            (PieceSet::RetroAscii, Role::Rook, false) => "[r]",
            (PieceSet::RetroAscii, Role::Bishop, false) => "[b]",
            (PieceSet::RetroAscii, Role::Knight, false) => "[n]",
            (PieceSet::RetroAscii, Role::Pawn, false) => "[p]",
            (_, Role::King, _) => "♚",
            (_, Role::Queen, _) => "♛",
            (_, Role::Rook, _) => "♜",
            (_, Role::Bishop, _) => "♝",
            (_, Role::Knight, _) => "♞",
            (_, Role::Pawn, _) => "♟",
        };

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            glyph,
            egui::FontId::monospace(sq_size * 0.42),
            text_col,
        );
    }
}

fn pos_to_square(pos: Pos2, board_origin: Pos2, sq_size: f32, flipped: bool) -> Option<Square> {
    let rel = pos - board_origin;
    if rel.x < 0.0 || rel.y < 0.0 { return None; }
    let file_idx = (rel.x / sq_size) as u32;
    let rank_idx = (rel.y / sq_size) as u32;
    if file_idx >= 8 || rank_idx >= 8 { return None; }

    let (display_file, display_rank) = if flipped {
        (7 - file_idx, rank_idx)
    } else {
        (file_idx, 7 - rank_idx)
    };

    Some(Square::from_coords(
        File::new(display_file),
        Rank::new(display_rank),
    ))
}
