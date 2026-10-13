use egui::{self, Align, Color32, Layout, Order, Rect, RichText, Vec2};
use crate::{AppScreen, LichefaceApp};
use crate::ui::board_widget::BoardWidget;

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

    // Request frequent repaints while game is active so the clock counts down smoothly in real time
    if app.game_info.is_some() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    let is_white_turn = app.board_state.turn() == shakmaty::Color::White;
    let (live_wtime, live_btime) = if let Some(state) = &app.game_state {
        let is_game_active = state.status == "started" || state.status == "created";
        let elapsed_ms = if is_game_active {
            app.clock_updated_at
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(0)
        } else {
            0
        };

        let w = if is_game_active && is_white_turn {
            state.wtime.saturating_sub(elapsed_ms)
        } else {
            state.wtime
        };

        let b = if is_game_active && !is_white_turn {
            state.btime.saturating_sub(elapsed_ms)
        } else {
            state.btime
        };

        (w, b)
    } else {
        (0, 0)
    };

    let (white_captured, black_captured, material_diff) = compute_material_and_captures(app.board_state.board());

    // Right side HUD Panel (responsive width and vertical scroll area)
    let hud_w = if flavor == crate::config::RetroFlavor::Cga { 280.0 } else { 265.0 };
    egui::SidePanel::right("game_hud_panel")
        .exact_width(hud_w)
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame::default()
            .fill(app.config.theme.panel_c(flavor))
            .stroke(if app.config.crt_effect { egui::Stroke::NONE } else { egui::Stroke::new(border_w, border_color) })
            .inner_margin(egui::Margin::symmetric(8.0, 8.0)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width());
                    ui.add_space(2.0);

            if let Some(info) = &app.game_info.clone() {
                let is_white_player = info.color == "white";
                let (my_color, opp_label) = if is_white_player {
                    ("Blancas", info.opponent.as_ref().map(|o| o.username.as_str()).unwrap_or("Oponente"))
                } else {
                    ("Negras", info.opponent.as_ref().map(|o| o.username.as_str()).unwrap_or("Oponente"))
                };
                let is_my_turn = if is_white_player { is_white_turn } else { !is_white_turn };
                let is_opp_turn = !is_my_turn;

                // Game Metadata Header (Speed, Time Control, Rated/Casual)
                let speed_label = match app.game_details.as_ref().map(|d| d.speed.as_str()) {
                    Some("bullet") | Some("ultraBullet") => "🚀 Bullet",
                    Some("blitz") => "⚡ Blitz",
                    Some("rapid") => "⏱ Rápida",
                    Some("classical") => "♟ Clásica",
                    _ => "⏱ Partida",
                };
                let time_ctrl = if let Some(d) = &app.game_details {
                    if d.initial_time_secs > 0 {
                        format!("{}+{}", d.initial_time_secs / 60, d.increment_secs)
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };
                let rated_label = if let Some(d) = &app.game_details {
                    if d.rated { "🏆 Clasificatoria" } else { "🤝 Amistosa" }
                } else {
                    "En línea"
                };

                let speed_badge_text = if time_ctrl.is_empty() {
                    speed_label.to_string()
                } else {
                    format!("{} {}", speed_label, time_ctrl)
                };

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                    egui::Frame::default()
                        .fill(Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 30))
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(1.0_f32, accent))
                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(speed_badge_text).color(accent).size(sz(10.5)).strong());
                        });

                    egui::Frame::default()
                        .fill(card_bg)
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(rated_label).color(text2).size(sz(10.0)));
                        });
                });

                ui.add_space(6.0);

                // Opponent ratings/titles
                let (opp_rating, opp_title) = if let Some(d) = &app.game_details {
                    if is_white_player {
                        (d.black_rating, d.black_title.as_deref())
                    } else {
                        (d.white_rating, d.white_title.as_deref())
                    }
                } else if let Some(opp) = &info.opponent {
                    (opp.rating, None)
                } else {
                    (None, None)
                };

                let (my_rating, my_title) = if let Some(d) = &app.game_details {
                    if is_white_player {
                        (d.white_rating, d.white_title.as_deref())
                    } else {
                        (d.black_rating, d.black_title.as_deref())
                    }
                } else {
                    (None, None)
                };

                // Captured pieces & material lead for each player
                let (opp_captured, my_captured, opp_lead, my_lead) = if is_white_player {
                    // Opponent is Black; captures White pieces
                    let opp_lead = if material_diff < 0 { -material_diff } else { 0 };
                    let my_lead = if material_diff > 0 { material_diff } else { 0 };
                    (&black_captured, &white_captured, opp_lead, my_lead)
                } else {
                    // Opponent is White; captures Black pieces
                    let opp_lead = if material_diff > 0 { material_diff } else { 0 };
                    let my_lead = if material_diff < 0 { -material_diff } else { 0 };
                    (&white_captured, &black_captured, opp_lead, my_lead)
                };

                // Opponent Card
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(rounding)
                    .stroke(egui::Stroke::new(border_w, if is_opp_turn { accent } else { border_color }))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🎭").size(sz(16.0)));
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if let Some(t) = opp_title {
                                        ui.label(RichText::new(format!("[{}]", t)).color(Color32::from_rgb(241, 196, 15)).size(sz(10.5)).strong());
                                    }
                                    let name_lbl = if retro { opp_label.to_uppercase() } else { opp_label.to_string() };
                                    ui.label(RichText::new(name_lbl).color(text).size(sz(11.5)).strong());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    if let Some(r) = opp_rating {
                                        ui.label(RichText::new(format!("({})", r)).color(text2).size(sz(10.5)));
                                    }
                                    let opp_col_label = if is_white_player { "Negras" } else { "Blancas" };
                                    ui.label(RichText::new(format!("• {}", opp_col_label)).color(text2).size(sz(10.0)));
                                });
                            });
                        });

                        ui.add_space(4.0);
                        if app.game_state.is_some() {
                            let opp_time_ms = if is_white_player { live_btime } else { live_wtime };
                            let opp_clock_color = if retro {
                                if is_opp_turn { accent } else { text2 }
                            } else if is_opp_turn {
                                accent
                            } else {
                                text
                            };
                            let clock_box_fill = if retro { card_bg } else { app.config.theme.bg2() };

                            egui::Frame::default()
                                .fill(clock_box_fill)
                                .rounding(rounding)
                                .stroke(egui::Stroke::new(border_w, if is_opp_turn { accent } else { border_color }))
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("⏱").size(sz(12.0)));
                                        ui.label(RichText::new(format_clock(opp_time_ms)).color(opp_clock_color).size(sz(17.0)).monospace().strong());
                                        if is_opp_turn {
                                            ui.label(RichText::new(if retro { "◄ TURNO" } else { "⏳" }).color(accent).size(sz(10.5)).strong());
                                        }
                                    });
                                });
                        }

                        // Captured tray
                        if !opp_captured.is_empty() || opp_lead > 0 {
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                let icons: String = opp_captured.iter().copied().collect();
                                ui.label(RichText::new(icons).color(text2).size(sz(11.0)));
                                if opp_lead > 0 {
                                    ui.label(RichText::new(format!("+{}", opp_lead)).color(accent).size(sz(10.5)).strong());
                                }
                            });
                        }
                    });

                ui.add_space(6.0);

                // Move History Card
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(rounding)
                    .stroke(egui::Stroke::new(border_w, border_color))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let moves_title = if retro { "[ REGISTRO DE JUGADAS ]" } else { "📜 Registro de Jugadas" };
                        ui.label(RichText::new(moves_title).color(text).size(sz(11.5)).strong());
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(140.0)
                            .stick_to_bottom(true)
                            .id_salt("game_moves_scroll")
                            .show(ui, |ui| {
                                let pairs: Vec<_> = app.san_history.chunks(2).collect();
                                if pairs.is_empty() {
                                    ui.label(RichText::new("Aún no hay jugadas...").color(text2).size(sz(10.5)));
                                }
                                for (i, pair) in pairs.iter().enumerate() {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.set_min_height(16.0);
                                        ui.label(RichText::new(format!("{}.", i + 1)).color(text2).size(sz(10.0)).monospace());
                                        ui.label(RichText::new(pair[0].as_str()).color(text).size(sz(10.5)).monospace().strong());
                                        if pair.len() > 1 {
                                            ui.label(RichText::new(pair[1].as_str()).color(text).size(sz(10.5)).monospace().strong());
                                        }
                                    });
                                }
                            });
                    });

                ui.add_space(6.0);

                // My Card
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(rounding)
                    .stroke(egui::Stroke::new(border_w, if is_my_turn { accent } else { border_color }))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let my_name = app.profile.as_ref().map(|p| p.username.as_str()).unwrap_or("Tú");
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("👤").size(sz(16.0)));
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if let Some(t) = my_title {
                                        ui.label(RichText::new(format!("[{}]", t)).color(Color32::from_rgb(241, 196, 15)).size(sz(10.5)).strong());
                                    }
                                    let display_name = if retro { my_name.to_uppercase() } else { my_name.to_string() };
                                    ui.label(RichText::new(display_name).color(text).size(sz(11.5)).strong());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    if let Some(r) = my_rating {
                                        ui.label(RichText::new(format!("({})", r)).color(text2).size(sz(10.5)));
                                    }
                                    ui.label(RichText::new(format!("• {}", my_color)).color(text2).size(sz(10.0)));
                                });
                            });
                        });

                        ui.add_space(4.0);
                        if app.game_state.is_some() {
                            let my_time_ms = if is_white_player { live_wtime } else { live_btime };
                            let my_clock_color = if retro {
                                if is_my_turn { accent } else { text2 }
                            } else if is_my_turn {
                                accent
                            } else {
                                text
                            };
                            let clock_box_fill = if retro { card_bg } else { app.config.theme.bg2() };

                            egui::Frame::default()
                                .fill(clock_box_fill)
                                .rounding(rounding)
                                .stroke(egui::Stroke::new(border_w, if is_my_turn { accent } else { border_color }))
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("⏱").size(sz(12.0)));
                                        ui.label(RichText::new(format_clock(my_time_ms)).color(my_clock_color).size(sz(17.0)).monospace().strong());
                                        if is_my_turn {
                                            ui.label(RichText::new(if retro { "◄ TU TURNO" } else { "🟢" }).color(accent).size(sz(10.5)).strong());
                                        }
                                    });
                                });
                        }

                        // Captured tray
                        if !my_captured.is_empty() || my_lead > 0 {
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                let icons: String = my_captured.iter().copied().collect();
                                ui.label(RichText::new(icons).color(text2).size(sz(11.0)));
                                if my_lead > 0 {
                                    ui.label(RichText::new(format!("+{}", my_lead)).color(accent).size(sz(10.5)).strong());
                                }
                            });
                        }
                    });

                ui.add_space(6.0);

                // Game Result Status Banner
                if let Some(state) = &app.game_state {
                    if state.status != "started" && state.status != "created" {
                        let result_msg = match state.winner.as_deref() {
                            Some("white") => "🏆 Ganador: Blancas".to_string(),
                            Some("black") => "🏆 Ganador: Negras".to_string(),
                            _ => format!("Resultado: {}", state.status),
                        };
                        egui::Frame::default()
                            .fill(Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 30))
                            .rounding(rounding)
                            .stroke(egui::Stroke::new(1.0_f32, accent))
                            .inner_margin(egui::Margin::same(6.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new(result_msg).color(accent).size(sz(11.5)).monospace().strong());
                            });
                        ui.add_space(6.0);
                    }
                }

                // Action buttons with Resign Confirmation
                let confirm_resign_id = ui.id().with("confirm_resign_flag");
                let confirming_resign = ui.data_mut(|d| d.get_temp::<bool>(confirm_resign_id).unwrap_or(false));

                let mut do_resign = false;
                let mut cancel_resign = false;
                let mut request_resign = false;

                if confirming_resign {
                    egui::Frame::default()
                        .fill(card_bg)
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(border_w, accent))
                        .inner_margin(egui::Margin::same(6.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new("¿RENDIRSE? ¿SEGURO?").color(accent).size(sz(11.0)).monospace().strong());
                            ui.add_space(4.0);
                            ui.columns(2, |cols| {
                                let w0 = cols[0].available_width();
                                let yes_btn = egui::Button::new(RichText::new("[✓ SI]").color(card_bg).size(sz(10.5)).monospace().strong())
                                    .fill(accent)
                                    .stroke(egui::Stroke::new(border_w, accent))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(w0, 24.0));
                                if cols[0].add(yes_btn).clicked() {
                                    do_resign = true;
                                }

                                let w1 = cols[1].available_width();
                                let no_btn = egui::Button::new(RichText::new("[✕ NO]").color(text).size(sz(10.5)).monospace().strong())
                                    .fill(card_bg)
                                    .stroke(egui::Stroke::new(border_w, border_color))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(w1, 24.0));
                                if cols[1].add(no_btn).clicked() {
                                    cancel_resign = true;
                                }
                            });
                        });
                } else {
                    ui.columns(2, |cols| {
                        let w0 = cols[0].available_width();
                        let draw_btn = egui::Button::new(RichText::new("[TABLAS]").color(text).size(sz(10.5)).monospace().strong())
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(border_w, border_color))
                            .rounding(rounding)
                            .min_size(Vec2::new(w0, 26.0));
                        if cols[0].add(draw_btn).clicked() {
                            let tx = app.tx.clone();
                            let token = app.config.api_token.clone();
                            let game_id = info.id.clone();
                            app.runtime.spawn(async move {
                                if let Err(e) = crate::lichess::api::offer_draw(&token, &game_id, true).await {
                                    let _ = tx.send(crate::AppEvent::Error(format!("Error al ofrecer tablas: {e}")));
                                } else {
                                    let _ = tx.send(crate::AppEvent::Error("Oferta de tablas enviada".to_string()));
                                }
                            });
                        }

                        let w1 = cols[1].available_width();
                        let resign_btn = egui::Button::new(RichText::new("[RENDIRSE]").color(text).size(sz(10.5)).monospace().strong())
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(border_w, border_color))
                            .rounding(rounding)
                            .min_size(Vec2::new(w1, 26.0));
                        if cols[1].add(resign_btn).clicked() {
                            request_resign = true;
                        }
                    });
                }

                if do_resign {
                    let token = app.config.api_token.clone();
                    let game_id = info.id.clone();
                    let tx = app.tx.clone();
                    // Inmediatamente purgar de la lista local de partidas en curso
                    app.now_playing.retain(|g| g.game_id != game_id && !game_id.starts_with(&g.game_id));
                    app.runtime.spawn(async move {
                        let _ = crate::lichess::api::resign_game(&token, &game_id).await;
                        // Esperar a que el servidor de Lichess registre el abandono y recargar lista oficial
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        if let Ok(games) = crate::lichess::api::get_now_playing(&token).await {
                            let _ = tx.send(crate::AppEvent::NowPlayingLoaded(games));
                        }
                    });
                    ui.data_mut(|d| d.insert_temp(confirm_resign_id, false));
                    app.end_game();
                }
                if cancel_resign {
                    ui.data_mut(|d| d.insert_temp(confirm_resign_id, false));
                }
                if request_resign {
                    ui.data_mut(|d| d.insert_temp(confirm_resign_id, true));
                }

                ui.add_space(8.0);
                let back_btn = egui::Button::new(RichText::new("[ ← Volver al inicio ]").color(text).size(sz(11.5)).monospace().strong())
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(border_w, border_color))
                    .rounding(rounding);
                if ui.add(back_btn).clicked() {
                    app.end_game();
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("No hay partida activa").color(text2).size(sz(13.0)).monospace());
                    ui.add_space(12.0);
                    let home_btn = egui::Button::new(RichText::new("[ 🏠 Ir al inicio ]").color(text).size(sz(12.0)).monospace().strong())
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(border_w, border_color))
                        .rounding(rounding);
                    if ui.add(home_btn).clicked() {
                        app.screen = AppScreen::Home;
                    }
                });
            }
                }); // close ScrollArea
        });

    // Central Panel: Chess Board & Controls
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(bg).inner_margin(egui::Margin::symmetric(14.0, 10.0)))
        .show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Board sizing: responsive auto-fit default or user-customized size
                // Strictly respect available area so the board adapts to window resizing and never overlaps sidebars
                let max_allowed_h = (ui.available_height() - 65.0).max(180.0);
                let max_allowed_w = (ui.available_width() - 32.0).max(180.0);
                let max_fitting_size = max_allowed_h.min(max_allowed_w);
                let mut current_board_size = app.config.board_size
                    .unwrap_or(max_fitting_size.min(620.0))
                    .min(max_fitting_size) // Strictly prevent overflow into side panels
                    .clamp(180.0, 920.0);

                // Quick Toolbar above the board: Zoom -, Zoom +, Auto, Flip
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
                    let zoom_out = ui.add(
                        egui::Button::new(RichText::new("[-]")
                            .color(text)
                            .monospace()
                            .strong()
                            .size(12.5))
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(border_w, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(32.0, 24.0))
                    )
                    .on_hover_text("Reducir tamaño del tablero (-30px)");
                    if zoom_out.clicked() {
                        current_board_size = (current_board_size - 30.0).max(180.0);
                        app.config.board_size = Some(current_board_size);
                        app.config.save();
                    }

                    let zoom_in = ui.add(
                        egui::Button::new(RichText::new("[+]")
                            .color(text)
                            .monospace()
                            .strong()
                            .size(12.5))
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(border_w, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(32.0, 24.0))
                    )
                    .on_hover_text("Aumentar tamaño del tablero (+30px)");
                    if zoom_in.clicked() {
                        current_board_size = (current_board_size + 30.0).min(max_fitting_size).min(920.0);
                        app.config.board_size = Some(current_board_size);
                        app.config.save();
                    }

                    let reset_btn = ui.add(
                        egui::Button::new(RichText::new("[Auto]")
                            .color(text)
                            .monospace()
                            .strong()
                            .size(12.0))
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(border_w, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(56.0, 24.0))
                    )
                    .on_hover_text("Ajustar automáticamente al espacio de la ventana");
                    if reset_btn.clicked() {
                        app.config.board_size = None;
                        app.config.save();
                    }

                    ui.label(RichText::new("|").color(border_color));

                    let flip_btn = ui.add(
                        egui::Button::new(RichText::new("[Girar]")
                            .color(text)
                            .monospace()
                            .strong()
                            .size(12.0))
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(border_w, border_color))
                        .rounding(rounding)
                        .min_size(Vec2::new(60.0, 24.0))
                    )
                    .on_hover_text("Invertir orientación del tablero");
                    if flip_btn.clicked() {
                        app.board_flipped_runtime = !app.board_flipped_runtime;
                    }

                    ui.label(RichText::new(format!("{:.0}px", current_board_size)).color(text2).monospace().size(11.5));
                });

                ui.add_space(6.0);

                let last_move = if let Some(last_uci) = app.move_history.last() {
                    parse_last_move(last_uci)
                } else {
                    None
                };

                let is_black_player = app.game_info.as_ref().map(|i| i.color == "black").unwrap_or(false);
                let flipped = is_black_player ^ app.board_flipped_runtime;

                // Board Card Wrapper
                let card_response = egui::Frame::default()
                    .fill(card_bg)
                    .rounding(rounding)
                    .stroke(egui::Stroke::new(border_w, border_color))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        let is_white_player = app.game_info.as_ref().map(|i| i.color == "white").unwrap_or(true);
                        let my_color = if is_white_player { shakmaty::Color::White } else { shakmaty::Color::Black };
                        let is_game_active = app.game_state.as_ref().map(|s| s.status == "started" || s.status == "created").unwrap_or(true);
                        let can_interact = is_game_active && (if is_white_player { is_white_turn } else { !is_white_turn });

                        let mut widget = BoardWidget::new(
                            &app.board_state,
                            &mut app.selected_square,
                            &app.config.theme,
                            &app.config.piece_set,
                            app.config.show_coordinates,
                            last_move,
                            current_board_size,
                        )
                        .with_flavor(flavor)
                        .with_interaction(can_interact, Some(my_color));
                        widget.flipped = flipped;

                        if let Some(uci) = widget.show(ui) {
                            if let Some(info) = &app.game_info.clone() {
                                app.board_state.make_uci_move(&uci);
                                app.move_history.push(uci.clone());
                                app.san_history = crate::chess::board::uci_to_san_list(&app.move_history, app.game_info.as_ref().map(|i| i.fen.as_str()));
                                if app.config.sound_enabled {
                                    app.audio.play_player_move();
                                }
                                app.send_move(&info.id, &uci);
                            }
                        }
                    });

                // Interactive Bottom-Left Drag Resize Handle
                let board_rect = card_response.response.rect;
                let handle_size = Vec2::new(26.0, 26.0);
                let bl_handle_rect = Rect::from_min_size(
                    egui::pos2(board_rect.left(), board_rect.bottom() - handle_size.y),
                    handle_size,
                );
                let handle_id = ui.id().with("board_resize_handle_bottom_left");
                let handle_resp = ui.interact(bl_handle_rect, handle_id, egui::Sense::drag())
                    .on_hover_text("Arrastra desde la esquina inferior izquierda para redimensionar el tablero");

                if handle_resp.hovered() || handle_resp.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                }

                if handle_resp.dragged() {
                    let delta = handle_resp.drag_delta();
                    // Dragging left or down increases size; dragging right or up decreases size
                    let change = delta.y - delta.x;
                    let new_size = (current_board_size + change).clamp(180.0, max_fitting_size.max(260.0));
                    app.config.board_size = Some(new_size);
                }

                if handle_resp.drag_stopped() {
                    app.config.save();
                }

                // Render visual grip pattern on bottom-left corner
                let painter = ui.painter();
                let grip_color = if handle_resp.hovered() || handle_resp.dragged() {
                    accent
                } else {
                    Color32::from_rgba_premultiplied(text2.r(), text2.g(), text2.b(), 130)
                };
                for offset in [5.0, 10.0, 15.0] {
                    painter.line_segment(
                        [
                            egui::pos2(bl_handle_rect.left() + 4.0, bl_handle_rect.bottom() - offset),
                            egui::pos2(bl_handle_rect.left() + offset, bl_handle_rect.bottom() - 4.0),
                        ],
                        egui::Stroke::new(1.8_f32, grip_color),
                    );
                }
            });
        });

    // ── Game Over Overlay ─────────────────────────────────────────────────────
    let is_game_over = app.game_state.as_ref().map(|s| {
        s.status != "started" && s.status != "created"
    }).unwrap_or(false);

    if is_game_over && !app.game_over_dismissed {
        if let Some(state) = app.game_state.clone() {
            let is_white_player = app.game_info.as_ref().map(|i| i.color == "white").unwrap_or(true);
            let my_name = app.profile.as_ref().map(|p| p.username.as_str()).unwrap_or("Tú");

            let i_won = state.winner.as_deref() == Some(if is_white_player { "white" } else { "black" });
            let opp_won = state.winner.as_deref() == Some(if is_white_player { "black" } else { "white" });
            let is_draw = state.winner.is_none() && matches!(
                state.status.as_str(),
                "draw" | "stalemate" | "repetition" | "insufficient" | "fiftyMoves"
            );
            let is_aborted = state.status == "aborted";

            let (result_ascii, result_line1, use_lose_color): (&str, &str, bool) = if is_aborted {
                ("[ -- ]", "PARTIDA CANCELADA", false)
            } else if i_won {
                ("[WIN!]", "** VICTORIA **", false)
            } else if opp_won {
                ("[LOSE]", "-- DERROTA --", true)
            } else if is_draw {
                ("[===]", "== TABLAS ==", false)
            } else {
                ("[FIN]", "PARTIDA TERMINADA", false)
            };

            let reason_text = match state.status.as_str() {
                "mate"         => "por jaque mate",
                "resign"       => "por abandono",
                "outoftime"    => "por tiempo agotado",
                "stalemate"    => "por ahogado",
                "draw"         => "por acuerdo mutuo",
                "repetition"   => "por repetición de posición",
                "insufficient" => "por material insuficiente",
                "fiftyMoves"   => "por la regla de los 50 movimientos",
                "aborted"      => "la partida ha sido anulada",
                _              => "",
            };

            let winner_line = if is_aborted {
                String::new()
            } else if i_won {
                format!("¡Felicidades, {}!", my_name)
            } else if opp_won {
                let opp = app.game_info.as_ref()
                    .and_then(|i| i.opponent.as_ref())
                    .map(|o| o.username.as_str())
                    .unwrap_or("Oponente");
                format!("Ganó {}", opp)
            } else {
                String::new()
            };

            let result_color = if i_won { accent } else if use_lose_color { text2 } else { text };

            // Semi-transparent dark backdrop
            egui::Area::new(egui::Id::new("game_over_backdrop"))
                .order(Order::Background)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .interactable(false)
                .show(ctx, |ui| {
                    let screen = ctx.screen_rect();
                    ui.painter().rect_filled(screen, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 160));
                });

            // Centered modal card
            let mut go_home = false;
            let mut dismiss = false;
            egui::Area::new(egui::Id::new("game_over_modal"))
                .order(Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    egui::Frame::default()
                        .fill(card_bg)
                        .rounding(rounding)
                        .stroke(egui::Stroke::new(2.5_f32, result_color))
                        .inner_margin(egui::Margin::symmetric(36.0, 28.0))
                        .show(ui, |ui| {
                            ui.set_min_width(300.0);
                            ui.set_max_width(420.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(result_ascii)
                                        .color(result_color)
                                        .size(sz(28.0))
                                        .monospace()
                                        .strong(),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(result_line1)
                                        .color(result_color)
                                        .size(sz(22.0))
                                        .monospace()
                                        .strong(),
                                );
                                if !winner_line.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(&winner_line)
                                            .color(text)
                                            .size(sz(13.0))
                                            .monospace()
                                            .strong(),
                                    );
                                }
                                if !reason_text.is_empty() {
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(reason_text)
                                            .color(text2)
                                            .size(sz(11.0))
                                            .monospace(),
                                    );
                                }
                                ui.add_space(20.0);
                                ui.columns(2, |cols| {
                                    let w0 = cols[0].available_width();
                                    let stay_btn = egui::Button::new(
                                        RichText::new("[VER TABLERO]")
                                            .color(text)
                                            .size(sz(10.5))
                                            .monospace()
                                            .strong(),
                                    )
                                    .fill(card_bg)
                                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(w0, 30.0));
                                    if cols[0].add(stay_btn).clicked() { dismiss = true; }

                                    let w1 = cols[1].available_width();
                                    let home_btn = egui::Button::new(
                                        RichText::new("[IR AL INICIO]")
                                            .color(card_bg)
                                            .size(sz(10.5))
                                            .monospace()
                                            .strong(),
                                    )
                                    .fill(result_color)
                                    .stroke(egui::Stroke::new(1.5_f32, result_color))
                                    .rounding(rounding)
                                    .min_size(Vec2::new(w1, 30.0));
                                    if cols[1].add(home_btn).clicked() { go_home = true; }
                                });
                            });
                        });
                });

            if dismiss { app.game_over_dismissed = true; }
            if go_home { app.end_game(); }
        }
    }
}

fn compute_material_and_captures(board: &shakmaty::Board) -> (Vec<&'static str>, Vec<&'static str>, i32) {
    let starting_counts = [8usize, 2, 2, 2, 1]; // P, N, B, R, Q
    let roles_icons = ["♙", "♘", "♗", "♖", "♕"];
    let values = [1i32, 3, 3, 5, 9];

    let mut w_counts = [0usize; 5];
    let mut b_counts = [0usize; 5];

    for sq in shakmaty::Square::ALL {
        if let Some(piece) = board.piece_at(sq) {
            let role_idx = match piece.role {
                shakmaty::Role::Pawn => Some(0),
                shakmaty::Role::Knight => Some(1),
                shakmaty::Role::Bishop => Some(2),
                shakmaty::Role::Rook => Some(3),
                shakmaty::Role::Queen => Some(4),
                shakmaty::Role::King => None,
            };
            if let Some(idx) = role_idx {
                match piece.color {
                    shakmaty::Color::White => w_counts[idx] += 1,
                    shakmaty::Color::Black => b_counts[idx] += 1,
                }
            }
        }
    }

    let mut white_lost = [0usize; 5];
    let mut black_lost = [0usize; 5];
    let mut w_mat = 0i32;
    let mut b_mat = 0i32;

    for i in 0..5 {
        white_lost[i] = starting_counts[i].saturating_sub(w_counts[i]);
        black_lost[i] = starting_counts[i].saturating_sub(b_counts[i]);
        w_mat += w_counts[i] as i32 * values[i];
        b_mat += b_counts[i] as i32 * values[i];
    }

    // Pieces captured BY white = missing black pieces
    let mut white_captured = Vec::new();
    for (i, &cnt) in black_lost.iter().enumerate() {
        for _ in 0..cnt {
            white_captured.push(roles_icons[i]);
        }
    }

    // Pieces captured BY black = missing white pieces
    let mut black_captured = Vec::new();
    for (i, &cnt) in white_lost.iter().enumerate() {
        for _ in 0..cnt {
            black_captured.push(roles_icons[i]);
        }
    }

    let diff = w_mat - b_mat;
    (white_captured, black_captured, diff)
}

fn format_clock(ms: u64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    let cents = (ms % 1000) / 10;
    if mins == 0 {
        format!("{:02}:{:02}.{:02}", mins, secs, cents)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

fn parse_last_move(uci: &str) -> Option<(shakmaty::Square, shakmaty::Square)> {
    if uci.len() >= 4 {
        let from = uci[..2].parse::<shakmaty::Square>().ok()?;
        let to = uci[2..4].parse::<shakmaty::Square>().ok()?;
        Some((from, to))
    } else {
        None
    }
}
