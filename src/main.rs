mod config;
mod theme;
mod lichess;
mod chess;
mod ui;
mod audio;

use eframe::egui::{self, Color32, FontFamily, FontId, Pos2, RichText, Vec2};
use std::sync::mpsc;
use tokio::runtime::Runtime;

pub use config::{AppConfig, RetroFlavor};
pub use theme::Theme;

pub enum AppEvent {
    ProfileLoaded(lichess::types::UserProfile),
    GameStarted(lichess::types::GameInfo),
    GameDetailsLoaded(lichess::types::GameDetails),
    GameState(lichess::types::GameState),
    Error(String),
    AuthSuccess(String), // username
    AuthFail(String),
    NowPlayingLoaded(Vec<lichess::types::NowPlayingGame>),
    SeekFinished,
}

#[derive(PartialEq, Clone, Copy)]
pub enum AppScreen {
    Setup,
    Home,
    Game,
    Settings,
}

pub struct LichefaceApp {
    screen: AppScreen,
    config: AppConfig,
    runtime: Runtime,
    tx: mpsc::Sender<AppEvent>,
    rx: mpsc::Receiver<AppEvent>,
    // UI state
    token_input: String,
    status_msg: String,
    board_state: chess::BoardState,
    game_info: Option<lichess::types::GameInfo>,
    game_state: Option<lichess::types::GameState>,
    profile: Option<lichess::types::UserProfile>,
    seek_time: u32,
    seek_increment: u32,
    pub rated_seek: bool,
    pub is_seeking: bool,
    pub now_playing: Vec<lichess::types::NowPlayingGame>,
    pub ai_level: u8,
    pub ai_time: u32,
    pub ai_increment: u32,
    pub ai_color: String,
    pub move_history: Vec<String>,
    pub san_history: Vec<String>,
    pub selected_square: Option<shakmaty::Square>,
    pub clock_updated_at: Option<std::time::Instant>,
    pub game_details: Option<lichess::types::GameDetails>,
    seek_active: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub notification: Option<(String, std::time::Instant)>,
    pub board_flipped_runtime: bool,
    pub audio: audio::AudioManager,
    pub game_over_dismissed: bool,
}

impl LichefaceApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let config = AppConfig::load();
        config.theme.apply_to_ctx(&cc.egui_ctx, config.retro_flavor);
        if config.fullscreen {
            cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }

        let (tx, rx) = mpsc::channel();
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        let screen = if config.api_token.is_empty() {
            AppScreen::Setup
        } else {
            AppScreen::Home
        };

        let app = Self {
            screen,
            config,
            runtime,
            tx,
            rx,
            token_input: String::new(),
            status_msg: String::new(),
            board_state: chess::BoardState::default(),
            game_info: None,
            game_state: None,
            game_details: None,
            profile: None,
            seek_time: 10,
            seek_increment: 0,
            rated_seek: false,
            is_seeking: false,
            seek_active: None,
            now_playing: Vec::new(),
            ai_level: 3,
            ai_time: 3,
            ai_increment: 0,
            ai_color: "random".to_string(),
            move_history: Vec::new(),
            san_history: Vec::new(),
            selected_square: None,
            clock_updated_at: None,
            notification: None,
            board_flipped_runtime: false,
            audio: audio::AudioManager::new(),
            game_over_dismissed: false,
        };

        // If already configured, fetch profile and now playing immediately
        if !app.config.api_token.is_empty() {
            app.fetch_profile();
            app.load_now_playing();
        }

        app
    }

    fn fetch_profile(&self) {
        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        self.runtime.spawn(async move {
            match lichess::api::get_profile(&token).await {
                Ok(profile) => { let _ = tx.send(AppEvent::ProfileLoaded(profile)); }
                Err(e) => { let _ = tx.send(AppEvent::Error(e.to_string())); }
            }
        });
    }

    fn validate_token(&self, token: String) {
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            match lichess::api::get_profile(&token).await {
                Ok(profile) => {
                    let _ = tx.send(AppEvent::AuthSuccess(profile.username.clone()));
                    let _ = tx.send(AppEvent::ProfileLoaded(profile));
                }
                Err(e) => { let _ = tx.send(AppEvent::AuthFail(e.to_string())); }
            }
        });
    }

    fn create_seek(&mut self) {
        self.is_seeking = true;
        let type_str = if self.rated_seek { "clasificatoria" } else { "amistosa" };
        self.status_msg = format!("Buscando oponente ({}+{}, {})...", self.seek_time, self.seek_increment, type_str);

        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        let time = self.seek_time;
        let inc = self.seek_increment;
        let rated = self.rated_seek;

        let seek_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        self.seek_active = Some(seek_active.clone());

        // Spawn stream seek
        let seek_tx = tx.clone();
        let seek_active_clone = seek_active.clone();
        let seek_token = token.clone();
        self.runtime.spawn(async move {
            match lichess::api::create_seek(&seek_token, time, inc, rated).await {
                Ok(game) => {
                    if seek_active_clone.load(std::sync::atomic::Ordering::Relaxed) {
                        let _ = seek_tx.send(AppEvent::GameStarted(game));
                    }
                }
                Err(e) => {
                    if seek_active_clone.load(std::sync::atomic::Ordering::Relaxed) {
                        let _ = seek_tx.send(AppEvent::Error(e.to_string()));
                        let _ = seek_tx.send(AppEvent::SeekFinished);
                    }
                }
            }
        });

        // Concurrently poll now_playing to catch the matched game immediately and switch screens
        let poller_tx = tx.clone();
        let poller_token = token.clone();
        let poller_active = seek_active.clone();
        self.runtime.spawn(async move {
            for _ in 0..60 { // Up to 90 seconds
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                if !poller_active.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                if let Ok(games) = lichess::api::get_now_playing(&poller_token).await {
                    if let Some(first) = games.into_iter().next() {
                        if poller_active.swap(false, std::sync::atomic::Ordering::Relaxed) {
                            let info = lichess::types::GameInfo {
                                id: first.game_id.clone(),
                                color: first.color,
                                fen: first.fen,
                                opponent: first.opponent,
                                seconds_left: first.seconds_left.unwrap_or(0.0),
                            };
                            let _ = poller_tx.send(AppEvent::GameStarted(info));
                            break;
                        }
                    }
                }
            }
        });
    }

    fn cancel_seek(&mut self) {
        if let Some(active) = self.seek_active.take() {
            active.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        self.is_seeking = false;
        self.status_msg = "Búsqueda cancelada.".to_string();
    }

    fn create_ai_game(&mut self, level: u8, time_mins: u32, inc_secs: u32, color: String) {
        self.status_msg = format!("Iniciando partida contra Stockfish (Nivel {})...", level);
        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        self.runtime.spawn(async move {
            match lichess::api::challenge_ai(&token, level, time_mins * 60, inc_secs, &color).await {
                Ok(game) => {
                    let _ = tx.send(AppEvent::GameStarted(game));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn load_now_playing(&self) {
        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        self.runtime.spawn(async move {
            if let Ok(games) = lichess::api::get_now_playing(&token).await {
                let _ = tx.send(AppEvent::NowPlayingLoaded(games));
            }
        });
    }

    fn resume_game(&mut self, game_id: String, color: String, fen: String) {
        let info = lichess::types::GameInfo {
            id: game_id.clone(),
            color,
            fen: fen.clone(),
            opponent: None,
            seconds_left: 0.0,
        };
        self.game_info = Some(info);
        self.game_state = None;          // clear any lingering finished state
        self.game_details = None;
        self.game_over_dismissed = false;
        self.board_state = chess::BoardState::from_fen(&fen);
        self.move_history.clear();
        self.san_history.clear();
        self.selected_square = None;
        self.clock_updated_at = Some(std::time::Instant::now());
        self.board_flipped_runtime = false;
        self.screen = AppScreen::Game;
        self.stream_game(game_id);
    }

    pub fn end_game(&mut self) {
        // Clear all game state so finished games don't linger in the UI
        if let Some(active) = self.seek_active.take() {
            active.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(info) = &self.game_info {
            let gid = info.id.clone();
            self.now_playing.retain(|g| g.game_id != gid && !gid.starts_with(&g.game_id));
        }
        self.game_info = None;
        self.game_state = None;
        self.game_details = None;
        self.move_history.clear();
        self.san_history.clear();
        self.selected_square = None;
        self.clock_updated_at = None;
        self.board_flipped_runtime = false;
        self.game_over_dismissed = false;
        self.screen = AppScreen::Home;
        // Refresh the now_playing list so the finished game disappears
        self.load_now_playing();
    }

    fn send_move(&self, game_id: &str, uci: &str) {
        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        let game_id = game_id.to_string();
        let uci = uci.to_string();
        self.runtime.spawn(async move {
            if let Err(e) = lichess::api::make_move(&token, &game_id, &uci).await {
                let _ = tx.send(AppEvent::Error(e.to_string()));
            }
        });
    }

    fn stream_game(&self, game_id: String) {
        let tx = self.tx.clone();
        let token = self.config.api_token.clone();
        self.runtime.spawn(async move {
            if let Err(e) = lichess::stream::stream_game(&token, &game_id, tx).await {
                eprintln!("Stream error: {e}");
            }
        });
    }

    pub fn show_notification(&mut self, msg: impl Into<String>) {
        self.notification = Some((msg.into(), std::time::Instant::now()));
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::ProfileLoaded(p) => {
                    self.status_msg = format!("Conectado como {}", p.username);
                    self.profile = Some(p);
                }
                AppEvent::AuthSuccess(username) => {
                    self.config.api_token = self.token_input.clone();
                    self.config.save();
                    self.status_msg = format!("¡Bienvenido, {}!", username);
                    self.screen = AppScreen::Home;
                }
                AppEvent::AuthFail(e) => {
                    self.status_msg = format!("Token inválido: {e}");
                }
                AppEvent::GameStarted(info) => {
                    if let Some(active) = self.seek_active.take() {
                        active.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                    self.is_seeking = false;
                    let game_id = info.id.clone();
                    self.game_info = Some(info);
                    self.board_state = chess::BoardState::default();
                    self.move_history.clear();
                    self.san_history.clear();
                    self.selected_square = None;
                    self.clock_updated_at = Some(std::time::Instant::now());
                    self.board_flipped_runtime = false;
                    self.game_over_dismissed = false;
                    self.screen = AppScreen::Game;
                    self.stream_game(game_id);
                }
                AppEvent::GameDetailsLoaded(details) => {
                    // Resolve true player color from definitive gameFull event
                    let mut resolved_color = None;

                    if details.white_is_ai && !details.black_is_ai {
                        resolved_color = Some("black");
                    } else if details.black_is_ai && !details.white_is_ai {
                        resolved_color = Some("white");
                    } else if let Some(p) = &self.profile {
                        let my_id = p.id.to_lowercase();
                        let my_name = p.username.to_lowercase();
                        let white_id = details.white_id.as_deref().unwrap_or("").to_lowercase();
                        let white_name = details.white_name.to_lowercase();
                        let black_id = details.black_id.as_deref().unwrap_or("").to_lowercase();
                        let black_name = details.black_name.to_lowercase();

                        if (!white_id.is_empty() && white_id == my_id) || white_name == my_name {
                            resolved_color = Some("white");
                        } else if (!black_id.is_empty() && black_id == my_id) || black_name == my_name {
                            resolved_color = Some("black");
                        }
                    }

                    if let Some(col) = resolved_color {
                        if let Some(info) = &mut self.game_info {
                            info.color = col.to_string();
                            let opp_name = if col == "white" { &details.black_name } else { &details.white_name };
                            let opp_rating = if col == "white" { details.black_rating } else { details.white_rating };
                            info.opponent = Some(lichess::types::Opponent {
                                username: opp_name.clone(),
                                rating: opp_rating,
                            });
                        }
                    }
                    self.game_details = Some(details);
                }
                AppEvent::GameState(state) => {
                    if !state.moves.is_empty() {
                        let new_moves: Vec<String> = state.moves.split_whitespace().map(|s| s.to_string()).collect();

                        // Reproducir sonido si el rival acaba de mover
                        if new_moves.len() > self.move_history.len() && self.config.sound_enabled {
                            let is_white = self.game_info.as_ref().map(|i| i.color == "white").unwrap_or(true);
                            let is_opponent_turn = if is_white {
                                new_moves.len() % 2 == 0 // Blancas = jugador -> turno par = rival (negras)
                            } else {
                                new_moves.len() % 2 == 1 // Negras = jugador -> turno impar = rival (blancas)
                            };

                            if is_opponent_turn && (!self.move_history.is_empty() || new_moves.len() == 1) {
                                self.audio.play_opponent_move();
                            }
                        }

                        let pos = chess::BoardState::from_moves(&state.moves);
                        self.board_state = pos;
                        self.move_history = new_moves;
                        self.san_history = crate::chess::board::uci_to_san_list(&self.move_history, self.game_info.as_ref().map(|i| i.fen.as_str()));
                        self.selected_square = None;
                    }
                    self.clock_updated_at = Some(std::time::Instant::now());
                    if state.status != "started" && state.status != "created" {
                        if let Some(info) = &self.game_info {
                            let gid = info.id.clone();
                            self.now_playing.retain(|g| g.game_id != gid && !gid.starts_with(&g.game_id));
                        }
                    }
                    self.game_state = Some(state);
                }
                AppEvent::NowPlayingLoaded(games) => {
                    self.now_playing = games;
                }
                AppEvent::SeekFinished => {
                    self.is_seeking = false;
                }
                AppEvent::Error(e) => {
                    self.is_seeking = false;
                    self.show_notification(format!("Aviso: {e}"));
                    self.status_msg = format!("Error: {e}");
                    eprintln!("Error: {e}");
                }
            }
        }
        ctx.request_repaint();
    }
}

impl eframe::App for LichefaceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_events(ctx);

        // Atajo de teclado F11 para alternar pantalla completa
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.config.fullscreen = !self.config.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.config.fullscreen));
            self.config.save();
            self.show_notification(if self.config.fullscreen { "Pantalla completa activada [F11]" } else { "Modo ventana activado [F11]" });
        }

        // On Setup screen, show centered onboarding
        if self.screen == AppScreen::Setup {
            ui::setup::show(self, ctx);
            return;
        }

        // Persistent Desktop Layout: Left Sidebar + Central Area
        let retro_flavor = self.config.retro_flavor;

        // Chasis exterior de Televisor CRT Vintage (si está activado crt_effect)
        let tv_tube_rect = if self.config.crt_effect {
            Some(render_tv_cabinet_housing(ctx, retro_flavor))
        } else {
            None
        };

        let panel_bg = self.config.theme.panel_c(retro_flavor);
        let card_bg = self.config.theme.card_c(retro_flavor);
        let border_color = self.config.theme.border_c(retro_flavor);
        let accent = self.config.theme.accent_c(retro_flavor);
        let text = self.config.theme.text_c(retro_flavor);
        let text2 = self.config.theme.text2_c(retro_flavor);
        let rounding = egui::Rounding::ZERO;

        egui::SidePanel::left("main_sidebar")
            .exact_width(235.0)
            .resizable(false)
            .show_separator_line(true)
            .frame(egui::Frame::default()
                .fill(panel_bg)
                .stroke(if self.config.crt_effect { egui::Stroke::NONE } else { egui::Stroke::new(1.5_f32, border_color) })
                .inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
                ui.add_space(6.0);

                // Brand Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("♞").color(accent).size(26.0).monospace().strong());
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        let title = match retro_flavor {
                            RetroFlavor::Cga => "LICHEFACE '81",
                            RetroFlavor::Vga => "LICHEFACE '89",
                            RetroFlavor::Monochrome => "LICHEFACE '84",
                            RetroFlavor::GreenPhosphor => "LICHEFACE '82",
                        };

                        let subtitle = match retro_flavor {
                            RetroFlavor::Cga => "★ CGA RETRO ★",
                            RetroFlavor::Vga => "★ VGA 256-COLOR DOS ★",
                            RetroFlavor::Monochrome => "★ HERCULES 1-BIT B&N ★",
                            RetroFlavor::GreenPhosphor => "★ FOSFORO P1 VERDE ★",
                        };
                        ui.label(RichText::new(title).color(text).size(15.0).monospace().strong());
                        ui.label(RichText::new(subtitle).color(accent).size(9.5).monospace().strong());
                    });
                });

                ui.add_space(20.0);

                // Navigation Items
                let nav_items = [
                    (AppScreen::Home, "[ 🏠 INICIO ]", false),
                    (AppScreen::Game, "[ ⚔ PARTIDA ]", self.game_info.is_some()),
                    (AppScreen::Settings, "[ ⚙ AJUSTES ]", false),
                ];

                for (screen, label, has_badge) in nav_items {
                    let is_active = self.screen == screen;
                    let (_bg_color, stroke_color, text_color) = if is_active {
                        (accent, accent, Color32::BLACK)
                    } else {
                        (card_bg, border_color, text)
                    };

                    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), egui::Sense::click());
                    let is_hovered = response.hovered() && !is_active;

                    let fill = if is_active {
                        accent
                    } else if is_hovered {
                        self.config.theme.hovered_btn_c(retro_flavor)
                    } else {
                        card_bg
                    };

                    let stroke = egui::Stroke::new(
                        if is_active { 1.5_f32 } else { 1.0_f32 },
                        if is_active || is_hovered { accent } else { stroke_color },
                    );

                    ui.painter().rect_filled(rect, 0.0, fill);
                    ui.painter().rect_stroke(rect, 0.0, stroke);

                    // Direct painter text rendering: 100% clickable everywhere with zero pointer obstruction
                    let font_size = if retro_flavor == RetroFlavor::Cga { 11.5 } else { 13.0 };
                    let font_id = FontId::new(font_size, FontFamily::Monospace);
                    ui.painter().text(
                        Pos2::new(rect.min.x + 10.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        label,
                        font_id.clone(),
                        text_color,
                    );

                    if has_badge {
                        ui.painter().text(
                            Pos2::new(rect.max.x - 12.0, rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            "●",
                            font_id,
                            if is_active { Color32::BLACK } else { accent },
                        );
                    }

                    if response.clicked() {
                        self.screen = screen;
                    }
                    if is_hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    ui.add_space(5.0);
                }

                // Push bottom items down
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);

                    // 4-Architecture Quick Switcher Strip:
                    // In bottom_up layout, adding buttons first places them at the bottom
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                        let item_w = (ui.available_width() - 9.0) / 4.0;
                        let flavors = [
                            (RetroFlavor::Cga, "CGA"),
                            (RetroFlavor::Vga, "VGA"),
                            (RetroFlavor::Monochrome, "B&N"),
                            (RetroFlavor::GreenPhosphor, "P1"),
                        ];
                        for (flv, flv_lbl) in flavors {
                            let is_active_flv = self.config.retro_flavor == flv;
                            let btn = egui::Button::new(
                                RichText::new(flv_lbl)
                                    .size(10.5)
                                    .monospace()
                                    .strong()
                                    .color(if is_active_flv { Color32::BLACK } else { text })
                            )
                            .fill(if is_active_flv { accent } else { card_bg })
                            .stroke(egui::Stroke::new(if is_active_flv { 1.5_f32 } else { 1.0_f32 }, if is_active_flv { accent } else { border_color }))
                            .rounding(rounding)
                            .min_size(Vec2::new(item_w, 26.0));

                            if ui.add(btn).clicked() {
                                self.config.set_flavor(flv);
                                self.config.theme.apply_to_ctx(ctx, flv);
                                self.config.save();
                                self.show_notification(format!("Arquitectura {} activada", flv.name()));
                            }
                        }
                    });
                    ui.add_space(4.0);

                    // Adding label second in bottom_up places it right ABOVE the buttons!
                    ui.label(RichText::new("ARQUITECTURA DE VIDEO:").color(text2).size(10.0).monospace().strong());

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // User Profile Status Card
                    egui::Frame::default()
                        .fill(card_bg)
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(1.5_f32, border_color))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if let Some(p) = &self.profile {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("👤").size(16.0));
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(&p.username).color(text).size(12.0).monospace().strong());
                                        ui.label(RichText::new("ONLINE [OK]").color(accent).size(10.0).monospace().strong());
                                    });
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⚪").size(12.0));
                                    ui.label(RichText::new("OFFLINE").color(text2).size(11.0).monospace());
                                });
                            }
                        });

                    ui.add_space(8.0);

                    // Botón para salir de la aplicación
                    let exit_btn = egui::Button::new(
                        RichText::new("[ ✕ SALIR DEL PROGRAMA ]").color(text2).size(11.5).monospace().strong(),
                    )
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.0_f32, border_color))
                    .rounding(rounding);

                    if ui.add_sized([ui.available_width(), 32.0], exit_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

        // Main content viewport
        match self.screen {
            AppScreen::Setup => ui::setup::show(self, ctx),
            AppScreen::Home => ui::home::show(self, ctx),
            AppScreen::Game => ui::game::show(self, ctx),
            AppScreen::Settings => ui::settings::show(self, ctx),
        }

        // Auto-dismissing floating retro toast / notification popup (5 seconds)
        if let Some((msg, created_at)) = &self.notification {
            if created_at.elapsed().as_secs_f32() >= 5.0 {
                self.notification = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));

                let flv = self.config.retro_flavor;
                let card_bg = self.config.theme.card_c(flv);
                let accent = self.config.theme.accent_c(flv);
                let toast_y_offset = if self.config.crt_effect { -68.0_f32 } else { -28.0_f32 };

                egui::Area::new(egui::Id::new("notification_toast_popup"))
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, toast_y_offset))
                    .order(egui::Order::Foreground)
                    .interactable(false)
                    .show(ctx, |ui| {
                        egui::Frame::default()
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(1.5_f32, accent))
                            .rounding(egui::Rounding::ZERO)
                            .inner_margin(egui::Margin::symmetric(24.0, 10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⚙").color(accent).size(15.0));
                                    ui.add_space(4.0);
                                    ui.label(RichText::new(msg).color(text).size(12.5).monospace().strong());
                                });
                            });
                    });
            }
        }

        // Superposición de Tubo CRT Curvo (abombado, scanlines y reflejo sobre la pantalla)
        if self.config.crt_effect {
            if let Some(rect) = tv_tube_rect {
                render_crt_tube_overlay(ctx, rect, self.config.retro_flavor);
            }
        }
    }
}

fn render_tv_cabinet_housing(ctx: &egui::Context, flavor: crate::config::RetroFlavor) -> egui::Rect {
    let cabinet_base = match flavor {
        crate::config::RetroFlavor::GreenPhosphor => Color32::from_rgb(18, 26, 20),
        _ => Color32::from_rgb(28, 30, 34),
    };
    let cabinet_dark = Color32::from_rgb(12, 13, 16);
    let cabinet_highlight = Color32::from_rgb(58, 62, 70);

    // 1. Panel Superior del Mueble TV (34px)
    egui::TopBottomPanel::top("tv_cabinet_top")
        .exact_height(34.0_f32)
        .show_separator_line(false)
        .frame(
            egui::Frame::default()
                .fill(cabinet_base)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(16.0, 6.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("||||||||||||||||").color(cabinet_dark).size(11.0).monospace().strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("||||||||||||||||").color(cabinet_dark).size(11.0).monospace().strong());
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("📺 LICHEVISION COLOR CRT · MODEL 1984")
                                .color(Color32::from_rgb(175, 180, 190))
                                .size(11.0)
                                .monospace()
                                .strong(),
                        );
                    });
                });
            });
        });

    // 2. Panel Inferior del Mueble TV (48px)
    egui::TopBottomPanel::bottom("tv_cabinet_bottom")
        .exact_height(48.0_f32)
        .show_separator_line(false)
        .frame(
            egui::Frame::default()
                .fill(cabinet_base)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("||||||||||||||||||||||||||").color(cabinet_dark).size(12.0).monospace().strong());
                ui.add_space(8.0);
                ui.label(
                    RichText::new("RGB CRT DISPLAY · 60Hz")
                        .color(Color32::from_rgb(115, 120, 130))
                        .size(10.0)
                        .monospace(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        let led_color = Color32::from_rgb(80, 255, 80);
                        ui.label(RichText::new("●").color(led_color).size(14.0).strong());
                        ui.label(RichText::new("POWER").color(led_color).size(9.5).monospace().strong());
                    });
                    ui.add_space(14.0);

                    let btn_style = |text: &str| {
                        RichText::new(text).color(Color32::from_rgb(180, 185, 195)).size(10.0).monospace().strong()
                    };
                    let dial_frame = |ui: &mut egui::Ui, text: &str| {
                        egui::Frame::default()
                            .fill(cabinet_dark)
                            .stroke(egui::Stroke::new(1.0_f32, cabinet_highlight))
                            .rounding(egui::Rounding::same(2.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(btn_style(text));
                            });
                    };

                    dial_frame(ui, "[ VOL 🕪 ]");
                    ui.add_space(4.0);
                    dial_frame(ui, "[ CH 03 ]");
                    ui.add_space(4.0);
                    dial_frame(ui, "[ BRIGHT ]");
                });
            });
        });

    // 3. Paneles Laterales del Mueble TV (26px cada uno)
    egui::SidePanel::left("tv_cabinet_left")
        .exact_width(26.0_f32)
        .resizable(false)
        .show_separator_line(false)
        .frame(
            egui::Frame::default()
                .fill(cabinet_base)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |_ui| {});

    egui::SidePanel::right("tv_cabinet_right")
        .exact_width(26.0_f32)
        .resizable(false)
        .show_separator_line(false)
        .frame(
            egui::Frame::default()
                .fill(cabinet_base)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |_ui| {});

    ctx.available_rect()
}

fn render_crt_tube_overlay(ctx: &egui::Context, tube_rect: egui::Rect, flavor: crate::config::RetroFlavor) {
    if tube_rect.width() <= 0.0 || tube_rect.height() <= 0.0 {
        return;
    }

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("crt_tube_overlay_layer"),
    ));

    let width = tube_rect.width();
    let height = tube_rect.height();
    let center_x = tube_rect.center().x;
    let center_y = tube_rect.center().y;
    let half_w = width * 0.5_f32;
    let half_h = height * 0.5_f32;

    // ── 1. SCANLINES CON ABOMBAMIENTO SUAVE DE TUBO CRT ──
    let scanline_color = match flavor {
        crate::config::RetroFlavor::GreenPhosphor => Color32::from_rgba_premultiplied(0, 30, 0, 52),
        crate::config::RetroFlavor::Cga => Color32::from_rgba_premultiplied(0, 0, 0, 56),
        crate::config::RetroFlavor::Monochrome => Color32::from_rgba_premultiplied(0, 0, 0, 56),
        crate::config::RetroFlavor::Vga => Color32::from_rgba_premultiplied(0, 0, 38, 50),
    };

    let max_bulge = 10.0_f32; // Flecha de curvatura suave y limpia
    let step = 3.0_f32;
    let mut y = tube_rect.top();

    let num_segments = 10;
    while y <= tube_rect.bottom() {
        let norm_y = (y - center_y) / half_h;
        let bulge_factor = norm_y * max_bulge;

        let mut prev_pt = {
            let x = tube_rect.left();
            let dx_ratio = 1.0_f32 - ((x - center_x) / half_w).powi(2);
            let cur_y = y - bulge_factor * dx_ratio;
            egui::pos2(x, cur_y)
        };

        for seg in 1..=num_segments {
            let t = seg as f32 / num_segments as f32;
            let x = tube_rect.left() + t * width;
            let dx_ratio = (1.0_f32 - ((x - center_x) / half_w).powi(2)).max(0.0_f32);
            let cur_y = y - bulge_factor * dx_ratio;
            let cur_pt = egui::pos2(x, cur_y);

            painter.line_segment([prev_pt, cur_pt], egui::Stroke::new(1.0_f32, scanline_color));
            prev_pt = cur_pt;
        }

        y += step;
    }

    // ── 2. VIÑETEADO ESFÉRICO 3D (CRISTAL CURVADO HACIA EL INTERIOR) ──
    let vignette_color = match flavor {
        crate::config::RetroFlavor::GreenPhosphor => Color32::from_rgba_premultiplied(0, 20, 0, 42),
        _ => Color32::from_rgba_premultiplied(0, 0, 0, 45),
    };
    for i in 1..=6 {
        let inset = i as f32 * 4.5_f32;
        painter.rect_stroke(
            tube_rect.shrink(inset),
            20.0_f32,
            egui::Stroke::new(4.5_f32, vignette_color),
        );
    }

    // ── 3. MASCARA SOLIDA DE LAS 4 ESQUINAS EXTERIORES (CHASIS DE ENCAJE) ──
    let cabinet_corner = match flavor {
        crate::config::RetroFlavor::GreenPhosphor => Color32::from_rgb(18, 26, 20),
        _ => Color32::from_rgb(28, 30, 34),
    };
    let cr = 24.0_f32;
    let n_steps = 8;
    // Helper para rellenar de forma estanca el área exterior entre el vértice recto del chasis y la curva del tubo
    let mask_corner = |corner_v: egui::Pos2, arc_center: egui::Pos2, start_angle: f32, end_angle: f32| {
        for s in 0..n_steps {
            let a1 = start_angle + (end_angle - start_angle) * (s as f32 / n_steps as f32);
            let a2 = start_angle + (end_angle - start_angle) * ((s + 1) as f32 / n_steps as f32);
            let p1 = arc_center + egui::vec2(a1.cos() * cr, a1.sin() * cr);
            let p2 = arc_center + egui::vec2(a2.cos() * cr, a2.sin() * cr);
            painter.add(egui::Shape::convex_polygon(
                vec![corner_v, p1, p2],
                cabinet_corner,
                egui::Stroke::NONE,
            ));
        }
    };

    use std::f32::consts::PI;
    // Esquina superior izquierda
    mask_corner(tube_rect.min, tube_rect.min + egui::vec2(cr, cr), PI, 1.5 * PI);
    // Esquina superior derecha
    let tr_v = egui::pos2(tube_rect.right(), tube_rect.top());
    mask_corner(tr_v, tr_v + egui::vec2(-cr, cr), 1.5 * PI, 2.0 * PI);
    // Esquina inferior izquierda
    let bl_v = egui::pos2(tube_rect.left(), tube_rect.bottom());
    mask_corner(bl_v, bl_v + egui::vec2(cr, -cr), PI, 0.5 * PI);
    // Esquina inferior derecha
    let br_v = tube_rect.max;
    mask_corner(br_v, br_v + egui::vec2(-cr, -cr), 0.0, 0.5 * PI);

    // ── 4. JUNTA DE GOMA Y BISEL DEL MARCO DEL TUBO (RUBBER GASKET) ──
    let gasket_color = Color32::from_rgb(10, 11, 14);
    painter.rect_stroke(tube_rect.shrink(1.0_f32), cr, egui::Stroke::new(3.0_f32, gasket_color));
    painter.rect_stroke(tube_rect.shrink(3.5_f32), cr - 4.0_f32, egui::Stroke::new(2.0_f32, Color32::from_rgb(18, 19, 23)));
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LICHEFACE")
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([850.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "LICHEFACE",
        options,
        Box::new(|cc| Ok(Box::new(LichefaceApp::new(cc)))),
    ).expect("Failed to run eframe");
}
