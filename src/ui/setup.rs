use egui::{self, Align, Color32, Layout, RichText, Vec2};
use crate::LichefaceApp;

pub fn show(app: &mut LichefaceApp, ctx: &egui::Context) {
    let flv = app.config.retro_flavor;
    let bg = app.config.theme.bg_c(flv);
    let card_bg = app.config.theme.card_c(flv);
    let border_color = app.config.theme.border_c(flv);
    let accent = app.config.theme.accent_c(flv);
    let text = app.config.theme.text_c(flv);
    let text2 = app.config.theme.text2_c(flv);

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(bg))
        .show(ctx, |ui| {
            // Top corner flavor badge & selector
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    for f in [
                        crate::config::RetroFlavor::Cga,
                        crate::config::RetroFlavor::Vga,
                        crate::config::RetroFlavor::Monochrome,
                        crate::config::RetroFlavor::GreenPhosphor,
                    ] {
                        let is_sel = app.config.retro_flavor == f;
                        let btn = egui::Button::new(
                            RichText::new(f.badge())
                                .color(if is_sel { Color32::BLACK } else { text })
                                .size(11.0)
                                .monospace()
                                .strong(),
                        )
                        .fill(if is_sel { accent } else { card_bg })
                        .stroke(egui::Stroke::new(1.0_f32, if is_sel { accent } else { border_color }))
                        .rounding(egui::Rounding::ZERO);

                        if ui.add(btn).clicked() {
                            app.config.retro_flavor = f;
                            app.config.theme.apply_to_ctx(ctx, app.config.retro_flavor);
                            app.config.save();
                        }
                    }
                });
            });

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(40.0);

                // Logo / Title
                ui.label(
                    RichText::new("♟ LICHEFACE [RETRO EDITION]")
                        .color(accent)
                        .size(36.0)
                        .monospace()
                        .strong(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("SISTEMA GRÁFICO RETRO // {}", app.config.retro_flavor.name()))
                        .color(text2)
                        .size(13.0)
                        .monospace(),
                );
                ui.add_space(28.0);

                // Card frame
                let card_width = 460.0_f32;
                egui::Frame::default()
                    .fill(card_bg)
                    .rounding(egui::Rounding::ZERO)
                    .stroke(egui::Stroke::new(1.5_f32, border_color))
                    .inner_margin(egui::Margin::same(26.0))
                    .show(ui, |ui| {
                        ui.set_width(card_width);
                        ui.set_max_width(card_width);

                        ui.label(
                            RichText::new("🔑 ACCESO API LICHESS")
                                .color(text)
                                .size(15.0)
                                .monospace()
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Introduce tu token de acceso personal con permisos de juego (Board API):")
                                .color(text2)
                                .size(12.0)
                                .monospace(),
                        );
                        ui.add_space(8.0);

                        // Link to Lichess token page
                        if ui.hyperlink_to(
                            "🔗 [GENERAR TOKEN EN LICHESS.ORG]",
                            "https://lichess.org/account/oauth/token/create?scopes[]=board:play&description=LICHEFACE",
                        ).clicked() {
                            let _ = open::that("https://lichess.org/account/oauth/token/create?scopes[]=board:play&description=LICHEFACE");
                        }

                        ui.add_space(14.0);

                        let w = ui.available_width();
                        let token_field = egui::TextEdit::singleline(&mut app.token_input)
                            .hint_text("lip_xxxxxxxxxxxxxxxx")
                            .password(true)
                            .desired_width(w)
                            .font(egui::TextStyle::Monospace);
                        ui.add(token_field);

                        ui.add_space(16.0);

                        let login_btn = egui::Button::new(
                            RichText::new("▶ CONECTAR A LICHESS").color(Color32::BLACK).size(14.0).monospace().strong()
                        )
                        .fill(accent)
                        .stroke(egui::Stroke::new(1.5_f32, border_color))
                        .rounding(egui::Rounding::ZERO)
                        .min_size(Vec2::new(w, 40.0));

                        if ui.add(login_btn).clicked() {
                            if !app.token_input.is_empty() {
                                app.status_msg = "Verificando token con la API de Lichess...".to_string();
                                app.validate_token(app.token_input.clone());
                            }
                        }

                        // Status message
                        if !app.status_msg.is_empty() {
                            ui.add_space(12.0);
                            let color = if app.status_msg.starts_with("Token inválido") || app.status_msg.starts_with("Error") {
                                Color32::from_rgb(255, 85, 85)
                            } else {
                                Color32::from_rgb(85, 255, 85)
                            };
                            ui.label(RichText::new(&app.status_msg).color(color).size(12.0).monospace().strong());
                        }
                    });

                ui.add_space(20.0);
                ui.label(
                    RichText::new("🔒 Tu token se almacena localmente en config.json de forma segura.")
                        .color(text2)
                        .size(11.0)
                        .monospace(),
                );
            });
        });
}
