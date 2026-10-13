use serde::{Deserialize, Serialize};
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use crate::config::RetroFlavor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,

    // Board colors
    pub light_square: [u8; 3],
    pub dark_square: [u8; 3],
    pub last_move_color: [u8; 4],
    pub selected_color: [u8; 4],
    pub check_color: [u8; 4],
    pub legal_dot_color: [u8; 4],

    // UI colors
    pub bg_primary: [u8; 3],
    pub bg_secondary: [u8; 3],
    pub panel_bg: [u8; 3],
    pub card_bg: [u8; 3],
    pub border_color: [u8; 3],
    pub accent: [u8; 3],
    pub text_primary: [u8; 3],
    pub text_secondary: [u8; 3],

    // Typography & mode
    pub font_size: f32,
    pub dark_mode: bool,
}

impl Default for Theme {
    fn default() -> Self {
        presets()[0].clone()
    }
}

pub fn color32(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

pub fn color32a(rgba: [u8; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

impl Theme {
    pub fn light_sq(&self) -> Color32 { color32(self.light_square) }
    pub fn dark_sq(&self) -> Color32 { color32(self.dark_square) }
    pub fn last_move(&self) -> Color32 { color32a(self.last_move_color) }
    pub fn selected(&self) -> Color32 { color32a(self.selected_color) }
    pub fn check(&self) -> Color32 { color32a(self.check_color) }
    pub fn legal_dot(&self) -> Color32 { color32a(self.legal_dot_color) }

    pub fn bg(&self) -> Color32 { color32(self.bg_primary) }
    pub fn bg2(&self) -> Color32 { color32(self.bg_secondary) }
    pub fn panel(&self) -> Color32 { color32(self.panel_bg) }
    pub fn card(&self) -> Color32 { color32(self.card_bg) }
    pub fn border(&self) -> Color32 { color32(self.border_color) }
    pub fn accent(&self) -> Color32 { color32(self.accent) }
    pub fn text(&self) -> Color32 { color32(self.text_primary) }
    pub fn text2(&self) -> Color32 { color32(self.text_secondary) }

    pub fn set_dark_mode(&mut self, _dark: bool) {}

    pub fn bg2_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).6
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context, flavor: RetroFlavor) {
        update_retro_fonts(ctx, flavor);

        let mut style = (*ctx.style()).clone();
        let mut visuals = Visuals::dark();

        let (card, panel, border, accent, text, _text2, inactive_btn, hovered_btn, extreme_bg, active_text) = retro_palette(flavor);

        visuals.dark_mode = true;
        visuals.window_fill = card;
        visuals.panel_fill = panel;
        // Do NOT override globally to allow buttons with fills to use high-contrast text
        visuals.override_text_color = None;
        visuals.hyperlink_color = accent;
        visuals.window_stroke = Stroke::new(1.5_f32, border);

        let rounding = egui::Rounding::ZERO;
        visuals.window_rounding = rounding;
        visuals.menu_rounding = rounding;

        // Buttons and interactive widgets
        visuals.widgets.noninteractive.bg_fill = card;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.5_f32, border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
        visuals.widgets.noninteractive.rounding = rounding;

        visuals.widgets.inactive.bg_fill = inactive_btn;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.5_f32, border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
        visuals.widgets.inactive.rounding = rounding;

        visuals.widgets.hovered.bg_fill = hovered_btn;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.8_f32, accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
        visuals.widgets.hovered.rounding = rounding;

        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, active_text);
        visuals.widgets.active.rounding = rounding;

        visuals.widgets.open.bg_fill = card;
        visuals.widgets.open.bg_stroke = Stroke::new(1.5_f32, border);
        visuals.widgets.open.rounding = rounding;

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 100);
        visuals.selection.stroke = Stroke::new(1.5_f32, accent);
        visuals.extreme_bg_color = extreme_bg;

        style.visuals = visuals;

        // Typography: strictly Monospace for authentic 80s-90s PC feel
        let base = match flavor {
            RetroFlavor::Cga => self.font_size * 0.78, // PC Senior is wide/chunky; scale to ~11.0pt to prevent overflow
            RetroFlavor::Vga => self.font_size * 0.95, // Perfect DOS VGA 437 fits well at ~13.5pt
            _ => self.font_size,
        };
        let font_fam = FontFamily::Monospace;
        style.text_styles = [
            (TextStyle::Small, FontId::new(base * 0.82, font_fam.clone())),
            (TextStyle::Body, FontId::new(base, font_fam.clone())),
            (TextStyle::Monospace, FontId::new(base * 0.95, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(base * 0.92, font_fam.clone())),
            (TextStyle::Heading, FontId::new(base * 1.30, font_fam)),
        ].into();

        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(12.0);

        ctx.set_style(style);

        let zoom = (self.font_size / 14.0).clamp(0.75, 1.75);
        ctx.set_zoom_factor(zoom);
    }
}

pub fn presets() -> Vec<Theme> {
    vec![
        // Preset 0: VGA DOS 1987 (Predeterminado)
        Theme {
            name: "VGA DOS 1987".into(),
            light_square: [255, 255, 85],  // Bright VGA Yellow
            dark_square: [85, 255, 255],   // VGA Cyan (exact match with UI text & decorations)
            last_move_color: [255, 255, 255, 170],
            selected_color: [0, 0, 170, 255], // Auténtico Azul DOS sólido (#1)
            check_color: [255, 255, 255, 255], // Blanco puro (#FFFFFF) de la paleta VGA
            legal_dot_color: [0, 0, 100, 255],
            bg_primary: [0, 0, 170],
            bg_secondary: [0, 0, 136],
            panel_bg: [0, 0, 110],
            card_bg: [0, 0, 160],
            border_color: [170, 170, 170],
            accent: [255, 255, 85],
            text_primary: [255, 255, 255],
            text_secondary: [85, 255, 255],
            font_size: 14.0,
            dark_mode: true,
        },
        // Preset 1: CGA 1981 (IBM PC 5150)
        Theme {
            name: "CGA 1981".into(),
            light_square: [85, 255, 255],  // Electric Cyan
            dark_square: [170, 0, 170],    // Deep Magenta
            last_move_color: [255, 255, 255, 180],
            selected_color: [0, 0, 0, 255],      // Negro puro (#0 de paleta 1)
            check_color: [255, 255, 255, 255],  // Blanco puro (#3 de paleta 1)
            legal_dot_color: [0, 0, 0, 255],
            bg_primary: [0, 0, 0],
            bg_secondary: [0, 0, 0],
            panel_bg: [0, 0, 0],
            card_bg: [0, 0, 0],
            border_color: [255, 85, 255],
            accent: [85, 255, 255],
            text_primary: [255, 255, 255],
            text_secondary: [85, 255, 255],
            font_size: 14.0,
            dark_mode: true,
        },
        // Preset 2: Monocromo 1984 (Hercules / Mac 128K)
        Theme {
            name: "Monocromo 1984".into(),
            light_square: [255, 255, 255], // Pure White
            dark_square: [18, 18, 18],     // Stark Black
            last_move_color: [200, 200, 200, 180],
            selected_color: [255, 255, 255, 255],
            check_color: [255, 255, 255, 255],
            legal_dot_color: [128, 128, 128, 255],
            bg_primary: [0, 0, 0],
            bg_secondary: [15, 15, 15],
            panel_bg: [0, 0, 0],
            card_bg: [15, 15, 15],
            border_color: [255, 255, 255],
            accent: [255, 255, 255],
            text_primary: [255, 255, 255],
            text_secondary: [200, 200, 200],
            font_size: 14.0,
            dark_mode: true,
        },
        // Preset 3: Fósforo P1 1982 (MDA / IBM 5151)
        Theme {
            name: "Fósforo P1 1982".into(),
            light_square: [60, 230, 60],   // Glowing Green
            dark_square: [0, 45, 0],       // Deep CRT green
            last_move_color: [51, 255, 51, 180],
            selected_color: [102, 255, 102, 255],
            check_color: [102, 255, 102, 255],
            legal_dot_color: [0, 20, 0, 255],
            bg_primary: [3, 10, 3],
            bg_secondary: [8, 26, 8],
            panel_bg: [3, 10, 3],
            card_bg: [8, 26, 8],
            border_color: [0, 190, 0],
            accent: [51, 255, 51],
            text_primary: [102, 255, 102],
            text_secondary: [45, 220, 45],
            font_size: 14.0,
            dark_mode: true,
        },
        // Preset 4: Ámbar CRT 1983 (Monitor Ámbar P3)
        Theme {
            name: "Ámbar CRT 1983".into(),
            light_square: [255, 176, 0],   // Glowing Amber
            dark_square: [40, 20, 0],      // Deep Amber Black
            last_move_color: [255, 210, 0, 180],
            selected_color: [255, 190, 40, 255],
            check_color: [255, 210, 0, 255],
            legal_dot_color: [20, 10, 0, 255],
            bg_primary: [12, 6, 0],
            bg_secondary: [24, 12, 0],
            panel_bg: [12, 6, 0],
            card_bg: [28, 14, 0],
            border_color: [200, 130, 0],
            accent: [255, 180, 0],
            text_primary: [255, 200, 50],
            text_secondary: [220, 150, 30],
            font_size: 14.0,
            dark_mode: true,
        },
        // Preset 5: Commodore 64 1982
        Theme {
            name: "Commodore 64 1982".into(),
            light_square: [136, 126, 203], // C64 Light Blue
            dark_square: [64, 49, 141],    // C64 Dark Blue
            last_move_color: [255, 255, 119, 180],
            selected_color: [163, 206, 39, 200],
            check_color: [200, 72, 72, 220],
            legal_dot_color: [0, 0, 0, 110],
            bg_primary: [64, 49, 141],
            bg_secondary: [50, 38, 112],
            panel_bg: [50, 38, 112],
            card_bg: [75, 60, 160],
            border_color: [136, 126, 203],
            accent: [255, 255, 119],
            text_primary: [255, 255, 255],
            text_secondary: [136, 126, 203],
            font_size: 14.0,
            dark_mode: true,
        },
    ]
}

pub fn retro_palette(flavor: RetroFlavor) -> (Color32, Color32, Color32, Color32, Color32, Color32, Color32, Color32, Color32, Color32) {
    // Returns (card, panel, border, accent, text, text2, inactive_btn, hovered_btn, extreme_bg, active_text)
    match flavor {
        RetroFlavor::Cga => (
            Color32::from_rgb(0, 0, 0),        // card: pure black
            Color32::from_rgb(0, 0, 0),        // panel: pure black
            Color32::from_rgb(255, 85, 255),   // border: high-voltage magenta
            Color32::from_rgb(85, 255, 255),   // accent: electric cyan
            Color32::from_rgb(255, 255, 255),  // text: pure white (100% legible)
            Color32::from_rgb(85, 255, 255),   // text2: electric cyan (100% legible)
            Color32::from_rgb(0, 0, 0),        // inactive_btn: pure black
            Color32::from_rgb(0, 0, 0),        // hovered_btn: pure black (accented by border)
            Color32::from_rgb(0, 0, 0),        // extreme_bg: pure black
            Color32::BLACK,                    // active_text
        ),
        RetroFlavor::Vga => (
            Color32::from_rgb(0, 0, 150),      // card: classic DOS dialog blue
            Color32::from_rgb(0, 0, 100),      // panel: classic MS-DOS blue
            Color32::from_rgb(180, 180, 180),  // border: DOS silver/light gray
            Color32::from_rgb(255, 255, 85),   // accent: bright VGA yellow
            Color32::from_rgb(255, 255, 255),  // text: pure white (100% legible)
            Color32::from_rgb(85, 255, 255),   // text2: bright VGA cyan (100% legible)
            Color32::from_rgb(20, 20, 180),    // inactive_btn
            Color32::from_rgb(40, 40, 210),    // hovered_btn
            Color32::from_rgb(0, 0, 65),       // extreme_bg
            Color32::BLACK,                    // active_text
        ),
        RetroFlavor::Monochrome => (
            Color32::from_rgb(15, 15, 15),     // card: deep black
            Color32::from_rgb(0, 0, 0),        // panel: pitch black
            Color32::from_rgb(255, 255, 255),  // border: pure white
            Color32::from_rgb(255, 255, 255),  // accent: pure white
            Color32::from_rgb(255, 255, 255),  // text: pure white (100% legible)
            Color32::from_rgb(200, 200, 200),  // text2: crisp silver (100% legible)
            Color32::from_rgb(28, 28, 28),     // inactive_btn
            Color32::from_rgb(55, 55, 55),     // hovered_btn
            Color32::from_rgb(0, 0, 0),        // extreme_bg
            Color32::BLACK,                    // active_text
        ),
        RetroFlavor::GreenPhosphor => (
            Color32::from_rgb(8, 26, 8),       // card: dark CRT phosphor base
            Color32::from_rgb(3, 10, 3),       // panel: dark CRT vacuum
            Color32::from_rgb(0, 190, 0),      // border: glowing CRT scanline green
            Color32::from_rgb(51, 255, 51),    // accent: incandescent phosphor green
            Color32::from_rgb(102, 255, 102),  // text: phosphor green (100% legible)
            Color32::from_rgb(45, 220, 45),    // text2: bright CRT green (100% legible)
            Color32::from_rgb(12, 40, 12),     // inactive_btn
            Color32::from_rgb(18, 65, 18),     // hovered_btn
            Color32::from_rgb(2, 6, 2),        // extreme_bg
            Color32::BLACK,                    // active_text
        ),
    }
}

impl Theme {
    pub fn bg_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).1
    }
    pub fn card_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).0
    }
    pub fn panel_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).1
    }
    pub fn border_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).2
    }
    pub fn accent_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).3
    }
    pub fn text_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).4
    }
    pub fn text2_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).5
    }
    pub fn inactive_btn_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).6
    }
    pub fn hovered_btn_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).7
    }
    pub fn active_text_c(&self, flavor: RetroFlavor) -> Color32 {
        retro_palette(flavor).9
    }

    pub fn retro_light_sq(&self, flavor: RetroFlavor) -> Color32 {
        match flavor {
            RetroFlavor::Cga => Color32::from_rgb(85, 255, 255), // Cyan
            RetroFlavor::Vga => Color32::from_rgb(255, 255, 85), // Bright VGA Yellow
            RetroFlavor::Monochrome => Color32::from_rgb(255, 255, 255), // Pure White
            RetroFlavor::GreenPhosphor => Color32::from_rgb(60, 230, 60), // Glowing Green
        }
    }

    pub fn retro_dark_sq(&self, flavor: RetroFlavor) -> Color32 {
        match flavor {
            RetroFlavor::Cga => Color32::from_rgb(170, 0, 170), // Magenta
            RetroFlavor::Vga => Color32::from_rgb(85, 255, 255), // Cyan exacto de la interfaz
            RetroFlavor::Monochrome => Color32::from_rgb(18, 18, 18), // Stark Black
            RetroFlavor::GreenPhosphor => Color32::from_rgb(0, 45, 0), // Dark CRT green
        }
    }

    pub fn retro_check_sq(&self, flavor: RetroFlavor, is_light: bool) -> Color32 {
        match flavor {
            RetroFlavor::Vga => Color32::from_rgb(255, 255, 255), // Blanco puro sólido (#FFFFFF) de la paleta VGA
            RetroFlavor::Cga => Color32::from_rgb(255, 255, 255), // Blanco puro CGA (#3 de paleta 1)
            RetroFlavor::Monochrome => if is_light { Color32::from_rgb(0, 0, 0) } else { Color32::from_rgb(255, 255, 255) },
            RetroFlavor::GreenPhosphor => Color32::from_rgb(102, 255, 102), // Fósforo verde alta intensidad
        }
    }

    pub fn retro_selected_sq(&self, flavor: RetroFlavor, is_light: bool) -> Color32 {
        match flavor {
            RetroFlavor::Vga => Color32::from_rgb(0, 0, 170), // Azul DOS sólido auténtico (#1)
            RetroFlavor::Cga => Color32::from_rgb(0, 0, 0), // Negro puro CGA (#0 de paleta 1)
            RetroFlavor::Monochrome => if is_light { Color32::from_rgb(0, 0, 0) } else { Color32::from_rgb(255, 255, 255) },
            RetroFlavor::GreenPhosphor => if is_light { Color32::from_rgb(0, 45, 0) } else { Color32::from_rgb(60, 230, 60) },
        }
    }

    pub fn retro_legal_dot(&self, flavor: RetroFlavor, is_light: bool) -> Color32 {
        match flavor {
            RetroFlavor::Vga => Color32::from_rgb(0, 0, 100),
            RetroFlavor::Cga => if is_light { Color32::from_rgb(170, 0, 170) } else { Color32::from_rgb(85, 255, 255) },
            RetroFlavor::Monochrome => if is_light { Color32::from_rgb(0, 0, 0) } else { Color32::from_rgb(255, 255, 255) },
            RetroFlavor::GreenPhosphor => if is_light { Color32::from_rgb(0, 30, 0) } else { Color32::from_rgb(51, 255, 51) },
        }
    }

    pub fn frame(&self, fill: Color32, border: Color32) -> egui::Frame {
        egui::Frame::default()
            .fill(fill)
            .rounding(egui::Rounding::ZERO)
            .stroke(egui::Stroke::new(1.5_f32, border))
    }
}

pub fn update_retro_fonts(ctx: &egui::Context, flavor: RetroFlavor) {
    let mut fonts = FontDefinitions::default();

    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let local_fonts = format!(r"{}\Microsoft\Windows\Fonts", local_app_data);

    let (font_key, candidates): (&str, Vec<String>) = match flavor {
        RetroFlavor::Cga => (
            "pc_senior",
            vec![
                format!(r"{}\PcSeniorRegular-OEnd.ttf", local_fonts),
                r"C:\Windows\Fonts\PcSeniorRegular-OEnd.ttf".to_string(),
                r"C:\Windows\Fonts\pcsenior.ttf".to_string(),
                r"C:\Windows\Fonts\lucon.ttf".to_string(),
                r"C:\Windows\Fonts\consola.ttf".to_string(),
            ],
        ),
        RetroFlavor::Vga => (
            "perfect_dos_vga",
            vec![
                format!(r"{}\Perfect DOS VGA 437 Win.ttf", local_fonts),
                r"C:\Windows\Fonts\Perfect DOS VGA 437 Win.ttf".to_string(),
                r"C:\Windows\Fonts\perfect_dos_vga_437.ttf".to_string(),
                r"C:\Windows\Fonts\consola.ttf".to_string(),
                r"C:\Windows\Fonts\lucon.ttf".to_string(),
            ],
        ),
        RetroFlavor::Monochrome => (
            "cour",
            vec![
                r"C:\Windows\Fonts\cour.ttf".to_string(),
                r"C:\Windows\Fonts\lucon.ttf".to_string(),
            ],
        ),
        RetroFlavor::GreenPhosphor => (
            "cascadia",
            vec![
                r"C:\Windows\Fonts\CascadiaMono.ttf".to_string(),
                r"C:\Windows\Fonts\lucon.ttf".to_string(),
            ],
        ),
    };

    let mut loaded = false;
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(font_key.to_owned(), FontData::from_owned(data).into());
            loaded = true;
            break;
        }
    }

    if loaded {
        if let Some(fam) = fonts.families.get_mut(&FontFamily::Monospace) {
            fam.insert(0, font_key.to_owned());
        }
        if let Some(fam) = fonts.families.get_mut(&FontFamily::Proportional) {
            fam.insert(0, font_key.to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_retro_fonts() {
        let ctx = egui::Context::default();
        update_retro_fonts(&ctx, RetroFlavor::Cga);
        update_retro_fonts(&ctx, RetroFlavor::Vga);
        update_retro_fonts(&ctx, RetroFlavor::Monochrome);
        update_retro_fonts(&ctx, RetroFlavor::GreenPhosphor);
    }
}
