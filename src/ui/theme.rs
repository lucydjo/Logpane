//! Terminal-like theme: monospace typography, square corners, 1 px rules, low-contrast
//! surfaces and one spacing scale shared by every panel, menu and inset.

use eframe::egui::{self, Color32, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

/// Single spacing scale. Every margin and gap in the UI uses one of these values.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

/// One-pixel rules, everywhere.
pub const BORDER: f32 = 1.0;

pub const HEADER_HEIGHT: f32 = 40.0;
pub const CONTROL_HEIGHT: f32 = 28.0;
pub const ROW_HEIGHT: f32 = 54.0;
pub const ACTION_BAR_HEIGHT: f32 = 30.0;
pub const SIDEBAR_MIN: f32 = 240.0;
pub const SIDEBAR_MAX: f32 = 340.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light => Self::System,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::System,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => crate::strings::STRINGS.theme_system,
            Self::Dark => crate::strings::STRINGS.theme_dark,
            Self::Light => crate::strings::STRINGS.theme_light,
        }
    }

    /// Resolve to an actual dark/light choice.
    pub fn resolve(self, system_dark: bool) -> Self {
        match self {
            Self::System => {
                if system_dark {
                    Self::Dark
                } else {
                    Self::Light
                }
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub separator: Color32,
    pub accent: Color32,
    pub selection: Color32,
    pub hover: Color32,
    pub danger: Color32,
    pub scrollbar: Color32,
}

impl Palette {
    /// Low-contrast pairing: surfaces sit within a few values of each other, text stays
    /// comfortably readable.
    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark | ThemeMode::System => Self {
                dark: true,
                bg: Color32::from_rgb(0x12, 0x13, 0x14),
                panel: Color32::from_rgb(0x15, 0x16, 0x17),
                raised: Color32::from_rgb(0x19, 0x1B, 0x1C),
                text: Color32::from_rgb(0xD2, 0xD3, 0xCF),
                secondary: Color32::from_rgb(0x85, 0x88, 0x8A),
                separator: Color32::from_rgb(0x23, 0x25, 0x27),
                accent: Color32::from_rgb(0x8F, 0xA6, 0xB8),
                selection: Color32::from_rgb(0x26, 0x31, 0x3A),
                hover: Color32::from_rgb(0x1C, 0x1E, 0x1F),
                danger: Color32::from_rgb(0xC9, 0x8B, 0x80),
                scrollbar: Color32::from_rgb(0x2D, 0x30, 0x32),
            },
            ThemeMode::Light => Self {
                dark: false,
                bg: Color32::from_rgb(0xF5, 0xF5, 0xF2),
                panel: Color32::from_rgb(0xFA, 0xFA, 0xF8),
                raised: Color32::from_rgb(0xEF, 0xEF, 0xEB),
                text: Color32::from_rgb(0x25, 0x27, 0x2A),
                secondary: Color32::from_rgb(0x67, 0x6A, 0x6E),
                separator: Color32::from_rgb(0xE2, 0xE2, 0xDD),
                accent: Color32::from_rgb(0x5A, 0x71, 0x87),
                selection: Color32::from_rgb(0xDC, 0xE4, 0xEA),
                hover: Color32::from_rgb(0xEC, 0xEC, 0xE7),
                danger: Color32::from_rgb(0xA7, 0x5C, 0x50),
                scrollbar: Color32::from_rgb(0xC9, 0xCA, 0xC5),
            },
        }
    }

    pub fn rule(&self) -> Stroke {
        Stroke::new(BORDER, self.separator)
    }
}

/// Square, thin-ruled surface used by every inset, card, notice and modal.
pub fn surface(palette: &Palette, fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(palette.rule())
        .corner_radius(egui::CornerRadius::same(0))
}

pub fn margin(value: f32) -> Margin {
    let value = value as i8;
    Margin {
        left: value,
        right: value,
        top: value,
        bottom: value,
    }
}

pub fn margin_xy(x: f32, y: f32) -> Margin {
    Margin {
        left: x as i8,
        right: x as i8,
        top: y as i8,
        bottom: y as i8,
    }
}

/// Load a monospace system font and use it for every family, including proportional text.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for candidate in [
        "/System/Library/Fonts/SFNSMono.ttf",
        "/System/Library/Fonts/Menlo.ttc",
    ] {
        let Ok(bytes) = std::fs::read(candidate) else {
            continue;
        };
        let name = "logpane-mono".to_owned();
        fonts.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(list) = fonts.families.get_mut(&family) {
                list.insert(0, name.clone());
            }
        }
        add_fallback_font(&mut fonts);
        ctx.set_fonts(fonts);
        return;
    }
    tracing::warn!("no monospace system font found; falling back to egui fonts");
}

/// Monospace system fonts stop at Latin, Greek and Cyrillic: other scripts need a fallback.
fn add_fallback_font(fonts: &mut egui::FontDefinitions) {
    let Ok(bytes) = std::fs::read(crate::export::FALLBACK_FONT_PATH) else {
        tracing::warn!("no fallback system font found; some scripts will not display");
        return;
    };
    let name = "logpane-fallback".to_owned();
    fonts.font_data.insert(
        name.clone(),
        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push(name.clone());
        }
    }
}

/// Install the palette and the terminal-like metrics into the egui context.
pub fn apply(ctx: &egui::Context, palette: &Palette) {
    let mut visuals = if palette.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.dark_mode = palette.dark;
    visuals.panel_fill = palette.panel;
    visuals.window_fill = palette.panel;
    visuals.extreme_bg_color = palette.bg;
    visuals.faint_bg_color = palette.raised;
    visuals.override_text_color = Some(palette.text);
    visuals.weak_text_color = Some(palette.secondary);
    visuals.selection.bg_fill = palette.selection;
    visuals.selection.stroke = Stroke::new(BORDER, palette.accent);
    visuals.hyperlink_color = palette.accent;
    visuals.window_stroke = palette.rule();
    visuals.text_edit_bg_color = Some(palette.bg);
    visuals.code_bg_color = palette.raised;
    visuals.warn_fg_color = palette.danger;
    visuals.error_fg_color = palette.danger;

    // Square everything and remove every shadow.
    let square = egui::CornerRadius::same(0);
    visuals.window_corner_radius = square;
    visuals.menu_corner_radius = square;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;

    visuals.widgets.noninteractive.bg_fill = palette.panel;
    visuals.widgets.noninteractive.weak_bg_fill = palette.panel;
    visuals.widgets.noninteractive.bg_stroke = palette.rule();
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(BORDER, palette.text);
    visuals.widgets.noninteractive.corner_radius = square;

    visuals.widgets.inactive.bg_fill = palette.scrollbar;
    visuals.widgets.inactive.weak_bg_fill = palette.panel;
    visuals.widgets.inactive.bg_stroke = palette.rule();
    visuals.widgets.inactive.fg_stroke = Stroke::new(BORDER, palette.text);
    visuals.widgets.inactive.corner_radius = square;

    visuals.widgets.hovered.bg_fill = palette.raised;
    visuals.widgets.hovered.weak_bg_fill = palette.raised;
    visuals.widgets.hovered.bg_stroke = Stroke::new(BORDER, palette.separator);
    visuals.widgets.hovered.fg_stroke = Stroke::new(BORDER, palette.accent);
    visuals.widgets.hovered.corner_radius = square;

    visuals.widgets.active.bg_fill = palette.selection;
    visuals.widgets.active.weak_bg_fill = palette.selection;
    visuals.widgets.active.bg_stroke = Stroke::new(BORDER, palette.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(BORDER, palette.accent);
    visuals.widgets.active.corner_radius = square;

    visuals.widgets.open.bg_fill = palette.raised;
    visuals.widgets.open.weak_bg_fill = palette.raised;
    visuals.widgets.open.bg_stroke = Stroke::new(BORDER, palette.separator);
    visuals.widgets.open.fg_stroke = Stroke::new(BORDER, palette.text);
    visuals.widgets.open.corner_radius = square;

    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        // Labels outside text editors must never behave like inputs.
        style.interaction.selectable_labels = false;

        style.spacing.item_spacing = egui::vec2(SPACE_S, SPACE_XS);
        style.spacing.button_padding = egui::vec2(SPACE_S, SPACE_XS);
        style.spacing.interact_size.y = 26.0;
        style.spacing.window_margin = margin(SPACE_M);
        style.spacing.menu_margin = margin(SPACE_XS);
        style.spacing.indent = SPACE_M;
        style.spacing.extra_text_line_spacing = 2.5;
        style.spacing.scroll.floating = false;
        style.spacing.scroll.foreground_color = false;
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.handle_min_length = 24.0;
        style.spacing.scroll.bar_inner_margin = 2.0;

        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::new(12.5, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::new(10.5, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(15.5, FontFamily::Monospace));
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        );
    });
}
