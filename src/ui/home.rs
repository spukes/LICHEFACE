use egui::{self, Color32, RichText, Vec2};
use crate::{AppScreen, LichefaceApp};

pub fn show(app: &mut LichefaceApp, ctx: &egui::Context) {
    let flavor = app.config.retro_flavor;
    let bg = app.config.theme.bg_c(flavor);
    let card_bg = app.config.theme.card_c(flavor);
    let border_color = app.config.theme.border_c(flavor);
    let accent = app.config.theme.accent_c(flavor);
    let text = app.config.theme.text_c(flavor);
    let text2 = app.config.theme.text2_c(flavor);
    let rounding = egui::Rounding::ZERO;
    let border_w = 1.5_f32;
    let retro = true;
    let sz = |s: f32| if flavor == crate::config::RetroFlavor::Cga { (s * 0.78).round().max(9.0) } else { s };

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(bg).inner_margin(egui::Margin::symmetric(16.0, 14.0)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Header Banner
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        if let Some(p) = &app.profile {
                            let greeting = if retro {
                                format!("[ {} // JUGADOR: {} ]", flavor.badge(), p.username.to_uppercase())
                            } else {
                                format!("¡Hola, {}! 👋", p.username)
                            };
                            ui.label(RichText::new(greeting).color(text).size(sz(if retro { 18.0 } else { 22.0 })).strong());
                        } else {
                            let greeting = if retro {
                                format!("[ SISTEMA: LICHEFACE {} ]", flavor.badge())
                            } else {
                                "¡Bienvenido a LICHEFACE! 👋".to_string()
                            };
                            ui.label(RichText::new(greeting).color(text).size(sz(if retro { 18.0 } else { 22.0 })).strong());
                        }
                        let subtitle = if retro {
                            match flavor {
                                crate::config::RetroFlavor::Cga => "CGA GRAPHICS 1981 // SALA DE JUEGO Y COMANDOS",
                                crate::config::RetroFlavor::Vga => "VGA DOS 1987 // SALA DE JUEGO Y COMANDOS",
                                crate::config::RetroFlavor::Monochrome => "HERCULES MONOCHROME 1984 // SALA DE JUEGO Y COMANDOS",
                                crate::config::RetroFlavor::GreenPhosphor => "FOSFORO P1 1982 // SALA DE JUEGO Y COMANDOS",
                            }
                        } else {
                            "Tu panel de juego online de ajedrez"
                        };
                        ui.label(RichText::new(subtitle).color(if retro { accent } else { text2 }).size(sz(11.5)).strong());
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let refresh_txt = if retro { "[ 🔄 ACTUALIZAR ]" } else { "🔄 Actualizar" };
                        let refresh_btn = egui::Button::new(RichText::new(refresh_txt).color(text).size(sz(11.5)).strong())
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(border_w, border_color))
                            .rounding(rounding);
                        if ui.add(refresh_btn).clicked() {
                            app.load_now_playing();
                            app.fetch_profile();
                        }
                    });
                });

                ui.add_space(14.0);

                // Ongoing games alert banner (if any)
                let has_active_game = app.game_info.is_some();
                let has_now_playing = !app.now_playing.is_empty();

                if has_active_game || has_now_playing {
                    egui::Frame::default()
                        .fill(card_bg)
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(border_w, accent))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("♟").color(accent).size(sz(20.0)).strong());
                                ui.vertical(|ui| {
                                    let alert_title = "[ PARTIDAS EN CURSO EN LICHESS ]";
                                    ui.label(RichText::new(alert_title).color(text).size(sz(13.0)).monospace().strong());
                                    ui.add_space(4.0);

                                    if has_active_game {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                                            ui.label(RichText::new("• Tienes una partida activa en el tablero").color(text2).size(sz(11.5)).monospace());

                                            let resume_active_btn = egui::Button::new(
                                                RichText::new("[ ▶ VOLVER AL TABLERO ]")
                                                    .color(Color32::BLACK)
                                                    .size(sz(11.0))
                                                    .monospace()
                                                    .strong()
                                            )
                                            .fill(accent)
                                            .stroke(egui::Stroke::new(1.0_f32, border_color))
                                            .rounding(rounding);

                                            if ui.add(resume_active_btn).clicked() {
                                                app.screen = AppScreen::Game;
                                            }
                                        });
                                    }

                                    for g in &app.now_playing.clone() {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                                            let opp_name = g.opponent.as_ref().map(|o| o.username.as_str()).unwrap_or("Oponente");
                                            let turn_str = if g.is_my_turn { "¡Es tu turno!" } else { "Turno del rival" };
                                            ui.label(RichText::new(format!("• vs {} ({}) - {}", opp_name, g.color, turn_str)).color(text2).size(sz(11.5)).monospace());

                                            let resume_btn = egui::Button::new(
                                                RichText::new("[ ▶ REANUDAR ]")
                                                    .color(Color32::BLACK)
                                                    .size(sz(11.0))
                                                    .monospace()
                                                    .strong()
                                            )
                                            .fill(accent)
                                            .stroke(egui::Stroke::new(1.0_f32, border_color))
                                            .rounding(rounding);

                                            if ui.add(resume_btn).clicked() {
                                                app.resume_game(g.game_id.clone(), g.color.clone(), g.fen.clone());
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    ui.add_space(14.0);
                }

                // Rating Metric Cards Row
                if let Some(p) = &app.profile.clone() {
                    let metrics = if retro {
                        vec![
                            ("[ BULLET ]", p.perfs.bullet.rating),
                            ("[ BLITZ ]", p.perfs.blitz.rating),
                            ("[ RAPIDA ]", p.perfs.rapid.rating),
                            ("[ CLASICA ]", p.perfs.classical.rating),
                        ]
                    } else {
                        vec![
                            ("⚡ Bullet", p.perfs.bullet.rating),
                            ("🔥 Blitz", p.perfs.blitz.rating),
                            ("⏱ Rápida", p.perfs.rapid.rating),
                            ("♟ Clásica", p.perfs.classical.rating),
                        ]
                    };

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                        for (title, rating) in metrics {
                            egui::Frame::default()
                                .fill(card_bg)
                                .rounding(rounding)
                                .stroke(egui::Stroke::new(border_w, border_color))
                                .inner_margin(egui::Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.set_min_size(Vec2::new(sz(130.0), sz(58.0)));
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(title).color(text2).size(sz(11.0)).strong());
                                        ui.add_space(3.0);
                                        let rating_str = if rating > 0 { format!("{}", rating) } else { "—".to_string() };
                                        ui.label(RichText::new(rating_str).color(accent).size(sz(20.0)).strong());
                                    });
                                });
                        }
                    });

                    ui.add_space(16.0);
                }

                let show_matchmaking_card = |ui: &mut egui::Ui, app: &mut LichefaceApp| {
                    ui.group(|ui| {
                        ui.set_min_height(370.0);
                        ui.set_width(ui.available_width());

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚔").color(accent).size(sz(18.0)).strong());
                            ui.label(RichText::new("Partida Online (Lichess Lobby)").color(text).size(sz(15.0)).strong());
                        });
                        ui.label(RichText::new("Emparejamiento público oficial mediante la Board API").color(text2).size(sz(11.0)));
                        ui.add_space(12.0);

                        // Mode selector: Ranked vs Casual
                        ui.label(RichText::new("Modalidad:").color(text).size(sz(12.0)).strong());
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                            let half_w = ((ui.available_width() - 8.0) * 0.5).max(105.0);

                            let casual_selected = !app.rated_seek;
                            let casual_btn = egui::Button::new(
                                RichText::new("🤝  Amistosa")
                                    .color(if casual_selected { Color32::BLACK } else { text })
                                    .size(sz(11.5))
                                    .strong()
                            )
                            .fill(if casual_selected { accent } else { card_bg })
                            .stroke(egui::Stroke::new(1.0_f32, if casual_selected { accent } else { border_color }))
                            .rounding(rounding)
                            .min_size(Vec2::new(half_w, 32.0));

                            if ui.add(casual_btn).clicked() {
                                app.rated_seek = false;
                            }

                            let rated_selected = app.rated_seek;
                            let rated_btn = egui::Button::new(
                                RichText::new("🏆  Clasificatoria")
                                    .color(if rated_selected { Color32::BLACK } else { text })
                                    .size(sz(11.5))
                                    .strong()
                            )
                            .fill(if rated_selected { accent } else { card_bg })
                            .stroke(egui::Stroke::new(1.0_f32, if rated_selected { accent } else { border_color }))
                            .rounding(rounding)
                            .min_size(Vec2::new(half_w, 32.0));

                            if ui.add(rated_btn).clicked() {
                                app.rated_seek = true;
                            }
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("Ritmo oficial (Rápida / Clásica):").color(text).size(sz(12.0)).strong());
                        ui.add_space(6.0);

                        // Allowed presets for Board API Seeks
                        let presets = [
                            (10, 0, "10+0 Rápida"),
                            (15, 10, "15+10 Rápida"),
                            (20, 0, "20+0 Rápida"),
                            (30, 0, "30+0 Clásica"),
                        ];

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                            for (t, inc, label) in presets {
                                let is_selected = app.seek_time == t && app.seek_increment == inc;
                                let chip_bg = if is_selected { accent } else { card_bg };
                                let chip_text_color = if is_selected { Color32::BLACK } else { text };

                                let chip = egui::Button::new(RichText::new(label).color(chip_text_color).size(sz(11.0)).strong())
                                    .fill(chip_bg)
                                    .stroke(egui::Stroke::new(1.0_f32, if is_selected { accent } else { border_color }))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(sz(88.0), 30.0));

                                if ui.add(chip).clicked() {
                                    app.seek_time = t;
                                    app.seek_increment = inc;
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.label(RichText::new("ℹ La Board API de Lichess restringe búsquedas públicas a ritmo Rápido/Clásico.").color(text2).size(sz(10.5)));
                        ui.add_space(14.0);

                        // Matchmaking Actions
                        if app.is_seeking {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.add_space(4.0);
                                ui.label(RichText::new("Buscando oponente en la sala...").color(accent).size(sz(12.0)).strong());
                            });
                            ui.add_space(10.0);

                            let cancel_btn = egui::Button::new(
                                RichText::new("❌  Cancelar Búsqueda").color(Color32::WHITE).size(sz(12.5)).strong()
                            )
                            .fill(Color32::BLACK)
                            .stroke(egui::Stroke::new(1.0_f32, border_color))
                            .rounding(rounding)
                            .min_size(Vec2::new(ui.available_width(), 38.0));

                            if ui.add(cancel_btn).clicked() {
                                app.cancel_seek();
                            }
                        } else {
                            let mode_tag = if app.rated_seek { "Clasif." } else { "Amist." };
                            let seek_label = format!("🎯  Buscar Oponente ({}+{} • {})", app.seek_time, app.seek_increment, mode_tag);
                            let play_btn = egui::Button::new(RichText::new(seek_label).color(Color32::BLACK).size(sz(12.5)).strong())
                                .fill(accent)
                                .stroke(egui::Stroke::new(1.0_f32, border_color))
                                .rounding(rounding)
                                .min_size(Vec2::new(ui.available_width(), 40.0));

                            if ui.add(play_btn).clicked() {
                                app.create_seek();
                            }
                        }

                        if !app.status_msg.is_empty() {
                            ui.add_space(8.0);
                            ui.label(RichText::new(&app.status_msg).color(accent).size(sz(11.0)));
                        }
                    });
                };

                let show_ai_card = |ui: &mut egui::Ui, app: &mut LichefaceApp| {
                    ui.group(|ui| {
                        ui.set_min_height(370.0);
                        ui.set_width(ui.available_width());

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🤖").color(accent).size(sz(18.0)).strong());
                            ui.label(RichText::new("Jugar contra Stockfish (IA)").color(text).size(sz(15.0)).strong());
                        });
                        ui.label(RichText::new("Inicio instantáneo sin esperas • Admite Blitz, y Rápida").color(text2).size(sz(11.0)));
                        ui.add_space(12.0);

                        // Difficulty Level
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                            ui.label(RichText::new("Dificultad:").color(text).size(sz(12.0)).strong());
                            ui.add(egui::Slider::new(&mut app.ai_level, 1..=8).text("Nivel"));
                        });
                        let elo_approx = 800 + (app.ai_level as u32) * 200;
                        ui.label(RichText::new(format!("Fuerza estimada: ~{} Elo", elo_approx)).color(text2).size(sz(11.0)));

                        ui.add_space(10.0);

                        // Cadence presets for AI
                        ui.label(RichText::new("Ritmo de juego:").color(text).size(sz(12.0)).strong());
                        ui.add_space(6.0);
                        let ai_presets = [
                            (3, 0, "3+0 Blitz"),
                            (5, 3, "5+3 Blitz"),
                            (10, 0, "10+0 Rápida"),
                        ];
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                            for (t, inc, label) in ai_presets {
                                let is_selected = app.ai_time == t && app.ai_increment == inc;
                                let chip_bg = if is_selected { accent } else { card_bg };
                                let chip_text_color = if is_selected { Color32::BLACK } else { text };

                                let chip = egui::Button::new(RichText::new(label).color(chip_text_color).size(sz(11.0)).strong())
                                    .fill(chip_bg)
                                    .stroke(egui::Stroke::new(1.0_f32, if is_selected { accent } else { border_color }))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(sz(84.0), 30.0));

                                if ui.add(chip).clicked() {
                                    app.ai_time = t;
                                    app.ai_increment = inc;
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Color selection
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                            ui.label(RichText::new("Jugar con:").color(text).size(sz(12.0)).strong());
                            ui.radio_value(&mut app.ai_color, "random".to_string(), "🎲 Azar");
                            ui.radio_value(&mut app.ai_color, "white".to_string(), "⚪ Blancas");
                            ui.radio_value(&mut app.ai_color, "black".to_string(), "⚫ Negras");
                        });

                        ui.add_space(14.0);

                        let ai_play_btn = egui::Button::new(
                            RichText::new("⚡  Iniciar Partida vs Stockfish").color(Color32::BLACK).size(sz(12.5)).strong()
                        )
                        .fill(accent)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(ui.available_width(), 40.0));

                        if ui.add(ai_play_btn).clicked() {
                            app.create_ai_game(app.ai_level, app.ai_time, app.ai_increment, app.ai_color.clone());
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        let settings_shortcut_btn = egui::Button::new(
                            RichText::new("⚙  Configurar Arquitectura de Video").color(text).size(sz(11.5)).strong()
                        )
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(ui.available_width(), 34.0));

                        if ui.add(settings_shortcut_btn).clicked() {
                            app.screen = AppScreen::Settings;
                        }
                    });
                };

                // Main actions: stack vertically on narrow windows (< 660px) or dual-column on wide windows
                let avail_w = ui.available_width();
                if avail_w < 660.0 {
                    ui.vertical(|ui| {
                        show_matchmaking_card(ui, app);
                        ui.add_space(14.0);
                        show_ai_card(ui, app);
                    });
                } else {
                    ui.columns(2, |cols| {
                        show_matchmaking_card(&mut cols[0], app);
                        show_ai_card(&mut cols[1], app);
                    });
                }

                ui.add_space(20.0);
            });
        });
}
