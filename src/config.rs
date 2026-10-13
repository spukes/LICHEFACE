use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_token: String,
    pub theme: Theme,
    pub piece_set: PieceSet,
    pub show_coordinates: bool,
    pub flip_board: bool,
    pub animation_speed: f32,
    pub sound_enabled: bool,
    pub ui_scale: f32,
    #[serde(default)]
    pub board_size: Option<f32>,
    #[serde(default)]
    pub retro_mode: bool,
    #[serde(default)]
    pub retro_flavor: RetroFlavor,
    #[serde(default)]
    pub crt_effect: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetroFlavor {
    Cga,          // Cyan / Magenta / Black / White (IBM 1981)
    Vga,          // DOS Deep Blue / Gray / White / Yellow / Cyan (VGA 1987)
    Monochrome,   // Pure Stark Black & White (Hercules / Mac 1984)
    GreenPhosphor // CRT Phosphor Green P39 (IBM 5151 / MDA)
}

impl RetroFlavor {
    pub fn name(&self) -> &'static str {
        match self {
            RetroFlavor::Cga => "CGA (1981)",
            RetroFlavor::Vga => "VGA DOS (1987)",
            RetroFlavor::Monochrome => "B&N Monocromo (1984)",
            RetroFlavor::GreenPhosphor => "Fósforo P1 (1982)",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            RetroFlavor::Cga => "CGA",
            RetroFlavor::Vga => "VGA",
            RetroFlavor::Monochrome => "B&N",
            RetroFlavor::GreenPhosphor => "P1",
        }
    }

    pub fn default_piece_set(&self) -> PieceSet {
        match self {
            RetroFlavor::Cga => PieceSet::RetroCga,
            RetroFlavor::Vga => PieceSet::RetroVga,
            RetroFlavor::Monochrome => PieceSet::RetroMonochrome,
            RetroFlavor::GreenPhosphor => PieceSet::RetroGreenPhosphor,
        }
    }

    pub fn default_theme(&self) -> Theme {
        match self {
            RetroFlavor::Cga => crate::theme::presets()[1].clone(),
            RetroFlavor::Vga => crate::theme::presets()[0].clone(),
            RetroFlavor::Monochrome => crate::theme::presets()[2].clone(),
            RetroFlavor::GreenPhosphor => crate::theme::presets()[3].clone(),
        }
    }
}

impl Default for RetroFlavor {
    fn default() -> Self {
        RetroFlavor::Vga
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PieceSet {
    RetroCga,           // CGA High Contrast (Cyan / Magenta)
    RetroVga,           // MS-DOS Chessmaster 1990
    RetroMonochrome,    // 1-Bit B&N Puro (Mac 1984 / Hercules)
    RetroGreenPhosphor, // Terminal Fósforo Verde CRT
    RetroPixel8Bit,     // Pixel Art Arcade 8-Bit
    RetroAscii,         // Glifos Monoespaciados Terminal ASCII
    CBurnett,           // Clásico Tradicional
    Merida,             // Mérida Torneo
    Alpha,              // Alpha
    Cardinal,           // Cardinal
}

impl PieceSet {
    pub fn name(&self) -> &'static str {
        match self {
            PieceSet::RetroCga => "CGA Retro (Cyan / Magenta)",
            PieceSet::RetroVga => "VGA Chessmaster 1990",
            PieceSet::RetroMonochrome => "1-Bit Monocromo (1984)",
            PieceSet::RetroGreenPhosphor => "Fósforo P1 Terminal",
            PieceSet::RetroPixel8Bit => "Pixel Art 8-Bit Arcade",
            PieceSet::RetroAscii => "Glifos ASCII Terminal",
            PieceSet::CBurnett => "CBurnett Tradicional",
            PieceSet::Merida => "Mérida Torneo",
            PieceSet::Alpha => "Alpha Clásico",
            PieceSet::Cardinal => "Cardinal",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PieceSet::RetroCga => "Paleta clásica de 4 colores Cyan y Magenta de 1981",
            PieceSet::RetroVga => "Piezas estilo VGA Chessmaster 1990",
            PieceSet::RetroMonochrome => "Estilo Macintosh 1984 1-bit blanco y negro puro",
            PieceSet::RetroGreenPhosphor => "Estética de fósforo P39 de terminal verde IBM 5151",
            PieceSet::RetroPixel8Bit => "Sprites arcade de 8 bits con píxeles nítidos",
            PieceSet::RetroAscii => "Caracteres y glifos de texto de terminal MS-DOS",
            PieceSet::CBurnett => "Diseño vectorial clásico refinado",
            PieceSet::Merida => "Set de torneo internacional Mérida",
            PieceSet::Alpha => "Set estilizado contemporáneo Alpha",
            PieceSet::Cardinal => "Set elegante de alto contraste Cardinal",
        }
    }

    pub fn default_for_flavor(flavor: RetroFlavor) -> Self {
        match flavor {
            RetroFlavor::Cga => PieceSet::RetroCga,
            RetroFlavor::Vga => PieceSet::RetroVga,
            RetroFlavor::Monochrome => PieceSet::RetroMonochrome,
            RetroFlavor::GreenPhosphor => PieceSet::RetroGreenPhosphor,
        }
    }

    pub fn all_retro() -> Vec<PieceSet> {
        vec![
            PieceSet::RetroCga,
            PieceSet::RetroVga,
            PieceSet::RetroMonochrome,
            PieceSet::RetroGreenPhosphor,
            PieceSet::RetroPixel8Bit,
            PieceSet::RetroAscii,
            PieceSet::CBurnett,
            PieceSet::Merida,
            PieceSet::Alpha,
            PieceSet::Cardinal,
        ]
    }
}

impl std::fmt::Display for PieceSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            theme: Theme::default(),
            piece_set: PieceSet::RetroVga,
            show_coordinates: true,
            flip_board: false,
            animation_speed: 1.0,
            sound_enabled: true,
            ui_scale: 1.0,
            board_size: None,
            retro_mode: true,
            retro_flavor: RetroFlavor::Vga,
            crt_effect: false,
            fullscreen: false,
        }
    }
}

impl AppConfig {
    pub fn set_flavor(&mut self, flavor: RetroFlavor) {
        self.retro_flavor = flavor;
        self.piece_set = flavor.default_piece_set();
        self.theme = flavor.default_theme();
    }

    pub fn config_path() -> PathBuf {
        let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push("licheface");
        p.push("config.json");
        p
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut cfg: Self = if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };
        // El set de piezas y el tablero vienen predefinidos rígidamente por la paleta retro elegida
        cfg.piece_set = cfg.retro_flavor.default_piece_set();
        cfg.theme = cfg.retro_flavor.default_theme();
        cfg
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retro_piece_sets_serde_roundtrip() {
        for set in PieceSet::all_retro() {
            let mut config = AppConfig::default();
            config.piece_set = set.clone();
            
            let serialized = serde_json::to_string(&config).expect("Serialization failed");
            let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialization failed");

            assert_eq!(deserialized.piece_set, set);
        }
    }

    #[test]
    fn test_retro_flavor_serde_roundtrip() {
        for flavor in [RetroFlavor::Cga, RetroFlavor::Vga, RetroFlavor::Monochrome, RetroFlavor::GreenPhosphor] {
            let mut config = AppConfig::default();
            config.retro_mode = true;
            config.retro_flavor = flavor;

            let serialized = serde_json::to_string(&config).expect("Serialization failed");
            let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialization failed");

            assert!(deserialized.retro_mode);
            assert_eq!(deserialized.retro_flavor, flavor);
        }
    }

    #[test]
    fn test_retro_flavor_default_when_missing() {
        let config = AppConfig::default();
        let mut val = serde_json::to_value(&config).expect("Serialize to Value failed");
        if let serde_json::Value::Object(ref mut map) = val {
            map.remove("retro_flavor");
        }
        let serialized = serde_json::to_string(&val).expect("Serialize back failed");
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.retro_flavor, RetroFlavor::Vga);
    }

    #[test]
    fn test_set_flavor_syncs_pieces_and_theme() {
        let mut config = AppConfig::default();
        for flavor in [RetroFlavor::Cga, RetroFlavor::Vga, RetroFlavor::Monochrome, RetroFlavor::GreenPhosphor] {
            config.set_flavor(flavor);
            assert_eq!(config.retro_flavor, flavor);
            assert_eq!(config.piece_set, flavor.default_piece_set());
            assert_eq!(config.theme.name, flavor.default_theme().name);
        }
    }

    #[test]
    fn test_crt_effect_serde() {
        let mut config = AppConfig::default();
        assert!(!config.crt_effect);
        config.crt_effect = true;
        let serialized = serde_json::to_string(&config).expect("Serialize failed");
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialize failed");
        assert!(deserialized.crt_effect);
    }

    #[test]
    fn test_fullscreen_serde() {
        let mut config = AppConfig::default();
        assert!(!config.fullscreen);
        config.fullscreen = true;
        let serialized = serde_json::to_string(&config).expect("Serialize failed");
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialize failed");
        assert!(deserialized.fullscreen);
    }

    #[test]
    fn test_sound_enabled_serde() {
        let mut config = AppConfig::default();
        assert!(config.sound_enabled);
        config.sound_enabled = false;
        let serialized = serde_json::to_string(&config).expect("Serialize failed");
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Deserialize failed");
        assert!(!deserialized.sound_enabled);
    }
}


