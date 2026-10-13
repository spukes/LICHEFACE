use egui::{self, Color32, RichText, Vec2};
use crate::{AppScreen, LichefaceApp};
use crate::config::RetroFlavor;
use crate::theme;

pub fn show(app: &mut LichefaceApp, ctx: &egui::Context) {
    let flv = app.config.retro_flavor;
    let bg = app.config.theme.bg_c(flv);
    let card_bg = app.config.theme.card_c(flv);
    let border_color = app.config.theme.border_c(flv);
    let accent = app.config.theme.accent_c(flv);
    let text = app.config.theme.text_c(flv);
    let text2 = app.config.theme.text2_c(flv);

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(bg).inner_margin(egui::Margin::same(24.0)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("⚙ AJUSTES Y SISTEMA RETRO")
                                .color(text)
                                .size(24.0)
                                .monospace()
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Personalización de arquitectura gráfica de video, catálogo de piezas retro y tableros históricos")
                                .color(text2)
                                .size(12.5)
                                .monospace(),
                        );
                    });
                });

                ui.add_space(20.0);

                // ── SECCION 1: ARQUITECTURA GRÁFICA DE VIDEO RETRO ──
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(egui::Rounding::ZERO)
                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🕹").size(18.0));
                            ui.label(
                                RichText::new("ARQUITECTURA DE VIDEO VINTAGE (1980 - 1990)")
                                    .color(text)
                                    .size(15.5)
                                    .monospace()
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                egui::Frame::default()
                                    .fill(Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 35))
                                    .stroke(egui::Stroke::new(1.5_f32, accent))
                                    .rounding(egui::Rounding::ZERO)
                                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(format!("★ {} [ACTIVO] ★", app.config.retro_flavor.badge()))
                                                .color(accent)
                                                .size(11.5)
                                                .monospace()
                                                .strong(),
                                        );
                                    });
                            });
                        });
                        ui.label(
                            RichText::new("Paletas de color fidedignas de época, tipografía monoespaciada con antialiasing nítido y biselados rectos.")
                                .color(text2)
                                .size(12.0)
                                .monospace(),
                        );
                        ui.add_space(14.0);

                        // 4 Historical Retro Architecture Cards
                        let flavors_data = [
                            (
                                RetroFlavor::Vga,
                                "VGA DOS (1987)",
                                "MS-DOS / Chessmaster",
                                "Azul clásico de MS-DOS, menús Norton Commander y marcos plata con texto blanco y amarillo.",
                                vec![Color32::from_rgb(0, 0, 160), Color32::from_rgb(180, 180, 180), Color32::from_rgb(255, 255, 85), Color32::WHITE],
                            ),
                            (
                                RetroFlavor::Cga,
                                "CGA (1981)",
                                "IBM PC 5150",
                                "Paleta 1 de 4 colores de alto contraste: cyan eléctrico, magenta vibrante, negro y blanco puro.",
                                vec![Color32::from_rgb(85, 255, 255), Color32::from_rgb(255, 85, 255), Color32::BLACK, Color32::WHITE],
                            ),
                            (
                                RetroFlavor::Monochrome,
                                "Monocromo (1984)",
                                "Hercules / Macintosh 128K",
                                "Contraste absoluto de 1 bit: blanco y negro puro, sin degradados, estética terminal auténtica.",
                                vec![Color32::BLACK, Color32::WHITE, Color32::from_rgb(200, 200, 200)],
                            ),
                            (
                                RetroFlavor::GreenPhosphor,
                                "Fósforo Verde (1982)",
                                "IBM 5151 / Monitor Fósforo P1",
                                "Monitor de fósforo verde P1 con brillo y contraste retro.",
                                vec![Color32::from_rgb(3, 10, 3), Color32::from_rgb(0, 190, 0), Color32::from_rgb(51, 255, 51), Color32::from_rgb(102, 255, 102)],
                            ),
                        ];

                        for (flv_option, title, hardware, desc, swatches) in flavors_data {
                            let is_active = app.config.retro_flavor == flv_option;
                            let card_border = if is_active { accent } else { border_color };
                            let card_bg_item = if is_active {
                                Color32::from_rgba_premultiplied(accent.r() / 4, accent.g() / 4, accent.b() / 4, 40)
                            } else {
                                app.config.theme.bg2_c(flv)
                            };

                            let frame = egui::Frame::default()
                                .fill(card_bg_item)
                                .stroke(egui::Stroke::new(if is_active { 1.5_f32 } else { 1.0_f32 }, card_border))
                                .rounding(egui::Rounding::ZERO)
                                .inner_margin(egui::Margin::same(12.0));

                            let r = frame.show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    // Radio indicator
                                    let radio_icon = if is_active { "●" } else { "○" };
                                    let radio_color = if is_active { accent } else { text2 };
                                    ui.label(RichText::new(radio_icon).color(radio_color).size(16.0).monospace().strong());
                                    ui.add_space(8.0);

                                    // Left: Info, description & predefined assets
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(title).color(if is_active { accent } else { text }).size(14.5).monospace().strong());
                                            ui.label(RichText::new(format!("[ {} ]", hardware)).color(text2).size(11.0).monospace());
                                            if is_active {
                                                ui.label(RichText::new("★ SELECCIONADO").color(accent).size(11.0).monospace().strong());
                                            }
                                        });
                                        ui.label(RichText::new(desc).color(text2).size(11.5).monospace());
                                        ui.add_space(3.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("● Piezas: {}", flv_option.default_piece_set().name())).color(if is_active { text } else { text2 }).size(11.0).monospace());
                                            ui.label(RichText::new("|").color(border_color).size(11.0).monospace());
                                            ui.label(RichText::new(format!("Tablero: {}", flv_option.default_theme().name)).color(if is_active { text } else { text2 }).size(11.0).monospace());
                                        });
                                    });

                                    // Right: Mini board preview, Swatches & Action Button
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let btn_label = if is_active { "✓ ACTIVO" } else { "Activar" };
                                        let btn = egui::Button::new(
                                            RichText::new(btn_label)
                                                .color(if is_active { Color32::BLACK } else { text })
                                                .size(11.5)
                                                .monospace()
                                                .strong()
                                        )
                                        .fill(if is_active { accent } else { card_bg })
                                        .stroke(egui::Stroke::new(if is_active { 1.5_f32 } else { 1.0_f32 }, if is_active { accent } else { border_color }))
                                        .rounding(egui::Rounding::ZERO)
                                        .min_size(Vec2::new(96.0, 30.0));

                                        if ui.add(btn).clicked() && !is_active {
                                            app.config.set_flavor(flv_option);
                                            app.config.theme.apply_to_ctx(ctx, app.config.retro_flavor);
                                            app.config.save();
                                            app.show_notification(format!("Arquitectura {} activada", flv_option.name()));
                                        }

                                        ui.add_space(8.0);

                                        // 4x4 Mini Board preview of the bound retro theme
                                        let target_theme = flv_option.default_theme();
                                        let sq = 7.0_f32;
                                        let board_size = Vec2::new(sq * 4.0, sq * 4.0);
                                        let (resp, painter) = ui.allocate_painter(board_size, egui::Sense::hover());
                                        for r_idx in 0..4u8 {
                                            for f_idx in 0..4u8 {
                                                let is_light = (r_idx + f_idx) % 2 == 1;
                                                let col = if is_light { theme::color32(target_theme.light_square) } else { theme::color32(target_theme.dark_square) };
                                                painter.rect_filled(
                                                    egui::Rect::from_min_size(
                                                        resp.rect.min + Vec2::new(f_idx as f32 * sq, r_idx as f32 * sq),
                                                        Vec2::splat(sq),
                                                    ),
                                                    0.0,
                                                    col,
                                                );
                                            }
                                        }
                                        painter.rect_stroke(resp.rect, 0.0, egui::Stroke::new(1.0_f32, border_color));

                                        ui.add_space(6.0);

                                        // Swatches preview
                                        ui.horizontal(|ui| {
                                            for sw in swatches {
                                                let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                                                ui.painter().rect_filled(rect, 0.0, sw);
                                                ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, border_color));
                                            }
                                        });
                                    });
                                });
                            });

                            let interact = r.response.interact(egui::Sense::click());
                            if interact.clicked() && !is_active {
                                app.config.set_flavor(flv_option);
                                app.config.theme.apply_to_ctx(ctx, app.config.retro_flavor);
                                app.config.save();
                                app.show_notification(format!("Arquitectura {} activada", flv_option.name()));
                            }
                            if interact.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            ui.add_space(5.0);
                        }

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Font size and scale
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Escala tipográfica (píxeles monoespaciados):").color(text).size(13.0).monospace().strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.0} px", app.config.theme.font_size)).color(accent).size(13.0).monospace().strong());
                            });
                        });
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            let size_presets = [
                                (12.0_f32, "12px", "Compacto"),
                                (14.0_f32, "14px", "Estándar"),
                                (16.0_f32, "16px", "Grande"),
                                (18.0_f32, "18px", "Extra"),
                            ];
                            for (size_val, label, desc) in size_presets {
                                let is_curr = (app.config.theme.font_size - size_val).abs() < 0.5;
                                let btn = egui::Button::new(
                                    RichText::new(format!("{}  {}", label, desc))
                                        .color(if is_curr { Color32::BLACK } else { text })
                                        .size(12.0)
                                        .monospace()
                                        .strong()
                                )
                                .fill(if is_curr { accent } else { app.config.theme.bg2_c(flv) })
                                .stroke(egui::Stroke::new(if is_curr { 1.5_f32 } else { 1.0_f32 }, if is_curr { accent } else { border_color }))
                                .rounding(egui::Rounding::ZERO)
                                .min_size(Vec2::new(108.0, 32.0));

                                if ui.add(btn).clicked() && !is_curr {
                                    app.config.theme.font_size = size_val;
                                    app.config.theme.apply_to_ctx(ctx, app.config.retro_flavor);
                                    app.config.save();
                                    app.show_notification(format!("Escala fijada en {:.0}px", size_val));
                                }
                                ui.add_space(8.0);
                            }
                        });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Ajuste fino:").color(text2).size(12.0).monospace());
                            let slider_resp = ui.add(
                                egui::Slider::new(&mut app.config.theme.font_size, 11.0..=20.0)
                                    .suffix(" px")
                                    .step_by(1.0)
                            );
                            if slider_resp.drag_stopped() || (!slider_resp.dragged() && slider_resp.changed()) {
                                app.config.theme.apply_to_ctx(ctx, app.config.retro_flavor);
                                app.config.save();
                                app.show_notification(format!("Escala ajustada a {:.0}px", app.config.theme.font_size));
                            }
                        });
                    });

                ui.add_space(16.0);

                // ── SECCION 2: PREFERENCIAS DE PARTIDA ──
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(egui::Rounding::ZERO)
                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚔").size(18.0));
                            ui.label(
                                RichText::new("PREFERENCIAS DE PARTIDA")
                                    .color(text)
                                    .size(15.5)
                                    .monospace()
                                    .strong(),
                            );
                        });
                        ui.add_space(12.0);

                        let mut pref_changed = false;
                        if ui.checkbox(&mut app.config.show_coordinates, RichText::new("Mostrar coordenadas de tablero (a-h, 1-8)").color(text).size(13.0).monospace()).changed() {
                            pref_changed = true;
                        }
                        ui.add_space(6.0);
                        if ui.checkbox(&mut app.config.flip_board, RichText::new("Voltear perspectiva del tablero").color(text).size(13.0).monospace()).changed() {
                            pref_changed = true;
                        }
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut app.config.sound_enabled, RichText::new("Activar efectos sonoros retro").color(text).size(13.0).monospace()).changed() {
                                pref_changed = true;
                            }
                            if app.config.sound_enabled {
                                ui.add_space(8.0);
                                let test_btn = egui::Button::new(RichText::new("🔊 PROBAR SONIDO").color(accent).size(11.0).monospace())
                                    .fill(card_bg)
                                    .stroke(egui::Stroke::new(1.0_f32, border_color))
                                    .rounding(egui::Rounding::ZERO);
                                if ui.add(test_btn).clicked() {
                                    app.audio.play_player_move();
                                    app.show_notification("Reproduciendo sonido de prueba...");
                                }
                            }
                        });
                        ui.add_space(6.0);
                        if ui.checkbox(&mut app.config.crt_effect, RichText::new("📺 Efecto pantalla de tubo CRT (líneas de escaneo / scanlines)").color(text).size(13.0).monospace()).changed() {
                            pref_changed = true;
                        }
                        ui.add_space(6.0);
                        if ui.checkbox(&mut app.config.fullscreen, RichText::new("⛶ Modo pantalla completa (o pulsa F11)").color(text).size(13.0).monospace()).changed() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.config.fullscreen));
                            pref_changed = true;
                        }

                        if pref_changed {
                            app.config.save();
                            app.show_notification("Preferencias guardadas.");
                        }
                    });

                ui.add_space(16.0);

                // ── SECCION 3: CUENTA LICHESS Y API ──
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(egui::Rounding::ZERO)
                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔑").size(18.0));
                            ui.label(
                                RichText::new("CUENTA LICHESS Y CONEXION API")
                                    .color(text)
                                    .size(15.5)
                                    .monospace()
                                    .strong(),
                            );
                        });
                        ui.add_space(8.0);

                        if let Some(p) = &app.profile {
                            ui.label(RichText::new(format!("Conectado como: {}", p.username)).color(accent).size(13.5).monospace().strong());
                        } else {
                            ui.label(RichText::new("Token configurado").color(text2).size(13.0).monospace());
                        }

                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            let save_btn = egui::Button::new(
                                RichText::new("💾 GUARDAR AJUSTES").color(Color32::BLACK).size(13.5).monospace().strong()
                            )
                            .fill(accent)
                            .stroke(egui::Stroke::new(1.0_f32, border_color))
                            .rounding(egui::Rounding::ZERO)
                            .min_size(Vec2::new(160.0, 36.0));

                            if ui.add(save_btn).clicked() {
                                app.config.save();
                                app.show_notification("Ajustes guardados correctamente.");
                            }

                            ui.add_space(10.0);

                            let disconnect_btn = egui::Button::new(
                                RichText::new("Desconectar / Cambiar token").color(text).size(13.0).monospace()
                            )
                            .fill(app.config.theme.bg2_c(flv))
                            .stroke(egui::Stroke::new(1.0_f32, border_color))
                            .rounding(egui::Rounding::ZERO)
                            .min_size(Vec2::new(190.0, 36.0));

                            if ui.add(disconnect_btn).clicked() {
                                app.config.api_token = String::new();
                                app.config.save();
                                app.screen = AppScreen::Setup;
                            }
                        });
                    });

                ui.add_space(16.0);

                // ── SECCION 4: CREDITOS Y CONTACTO ──
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(egui::Rounding::ZERO)
                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📜").size(18.0));
                            ui.label(
                                RichText::new("CREDITOS Y CONTACTO")
                                    .color(text)
                                    .size(15.5)
                                    .monospace()
                                    .strong(),
                            );
                        });
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("LICHEFACE · Cliente de Ajedrez Retro Vintage")
                                .color(text)
                                .size(13.0)
                                .monospace()
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Autor:").color(text2).size(12.5).monospace());
                            ui.label(RichText::new("Trevizer").color(accent).size(12.5).monospace().strong());
                        });
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sitio Web / Juegos:").color(text2).size(12.0).monospace());
                            ui.hyperlink_to(
                                RichText::new("https://trevizer.itch.io/").color(accent).size(12.0).monospace().strong(),
                                "https://trevizer.itch.io/",
                            );
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Red Social X:").color(text2).size(12.0).monospace());
                            ui.hyperlink_to(
                                RichText::new("x.com/AjedrezGuy").color(accent).size(12.0).monospace().strong(),
                                "https://x.com/AjedrezGuy",
                            );
                        });
                    });

                ui.add_space(24.0);
            });
        });
}
