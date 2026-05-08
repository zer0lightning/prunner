use eframe::egui;

// ── Dark palette ──────────────────────────────────────────────────────────────
// Background layers
pub const DARK_BG:         egui::Color32 = egui::Color32::from_rgb(22,  22,  26 );  // main window
pub const DARK_BG_RAISED:  egui::Color32 = egui::Color32::from_rgb(30,  30,  36 );  // popup / card
pub const DARK_BORDER:     egui::Color32 = egui::Color32::from_rgb(55,  55,  65 );
pub const DARK_SEP:        egui::Color32 = egui::Color32::from_rgb(48,  48,  58 );
// Text
pub const DARK_TEXT:       egui::Color32 = egui::Color32::from_rgb(230, 230, 235);  // primary — near white
pub const DARK_TEXT_DIM:   egui::Color32 = egui::Color32::from_rgb(140, 140, 155);  // secondary
// Accent (Windows blue)
pub const ACCENT:          egui::Color32 = egui::Color32::from_rgb(0,   120, 212);
pub const ACCENT_HOVER:    egui::Color32 = egui::Color32::from_rgb(28,  141, 230);
// Controls
pub const DARK_BTN_HOVER:  egui::Color32 = egui::Color32::from_rgb(58,  58,  70 );
pub const DARK_BTN_BG:     egui::Color32 = egui::Color32::from_rgb(45,  45,  54 );
pub const DARK_INPUT_BG:   egui::Color32 = egui::Color32::from_rgb(18,  18,  22 );
// Selection / hover in dropdown
pub const DARK_SEL:        egui::Color32 = egui::Color32::from_rgb(0,   84,  170);
pub const DARK_HOVER:      egui::Color32 = egui::Color32::from_rgb(38,  38,  50 );
// Error
pub const ERROR:           egui::Color32 = egui::Color32::from_rgb(232, 80,  70 );

// ── Light palette ─────────────────────────────────────────────────────────────
pub const LIGHT_BG:         egui::Color32 = egui::Color32::from_rgb(245, 245, 248);
pub const LIGHT_BG_RAISED:  egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
pub const LIGHT_BORDER:     egui::Color32 = egui::Color32::from_rgb(200, 200, 210);
pub const LIGHT_SEP:        egui::Color32 = egui::Color32::from_rgb(210, 210, 220);
pub const LIGHT_TEXT:       egui::Color32 = egui::Color32::from_rgb(18,  18,  22 );
pub const LIGHT_TEXT_DIM:   egui::Color32 = egui::Color32::from_rgb(100, 100, 115);
pub const LIGHT_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(210, 210, 218);
pub const LIGHT_BTN_BG:     egui::Color32 = egui::Color32::from_rgb(228, 228, 234);
pub const LIGHT_INPUT_BG:   egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
pub const LIGHT_SEL:        egui::Color32 = egui::Color32::from_rgb(204, 228, 255);
pub const LIGHT_HOVER:      egui::Color32 = egui::Color32::from_rgb(232, 238, 248);

/// Convenience: pick dark or light colour based on mode
pub struct Colors {
    pub dark: bool,
    pub bg:           egui::Color32,
    pub bg_raised:    egui::Color32,
    pub border:       egui::Color32,
    pub sep:          egui::Color32,
    pub text:         egui::Color32,
    pub text_dim:     egui::Color32,
    pub btn_bg:       egui::Color32,
    pub input_bg:     egui::Color32,
    pub sel:          egui::Color32,
    pub hover:        egui::Color32,
}

impl Colors {
    pub fn from_ctx(ctx: &egui::Context) -> Self {
        let dark = ctx.style().visuals.dark_mode;
        if dark {
            Self {
                dark: true,
                bg:           DARK_BG,
                bg_raised:    DARK_BG_RAISED,
                border:       DARK_BORDER,
                sep:          DARK_SEP,
                text:         DARK_TEXT,
                text_dim:     DARK_TEXT_DIM,
                btn_bg:       DARK_BTN_BG,
                input_bg:     DARK_INPUT_BG,
                sel:          DARK_SEL,
                hover:        DARK_HOVER,
            }
        } else {
            Self {
                dark: false,
                bg:           LIGHT_BG,
                bg_raised:    LIGHT_BG_RAISED,
                border:       LIGHT_BORDER,
                sep:          LIGHT_SEP,
                text:         LIGHT_TEXT,
                text_dim:     LIGHT_TEXT_DIM,
                btn_bg:       LIGHT_BTN_BG,
                input_bg:     LIGHT_INPUT_BG,
                sel:          LIGHT_SEL,
                hover:        LIGHT_HOVER,
            }
        }
    }
}

pub fn apply(ctx: &egui::Context) {
    #[cfg(windows)]
    {
        if let Some(dark) = read_windows_dark_mode() {
            let v = build_visuals(dark);
            ctx.set_visuals(v);
            return;
        }
    }
    let dark = ctx.style().visuals.dark_mode;
    ctx.set_visuals(build_visuals(dark));
}

fn build_visuals(dark: bool) -> egui::Visuals {
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };

    if dark {
        v.panel_fill           = DARK_BG;
        v.window_fill          = DARK_BG;
        v.faint_bg_color       = DARK_BG_RAISED;
        v.extreme_bg_color     = DARK_INPUT_BG;
        v.widgets.noninteractive.bg_fill   = DARK_BG;
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, DARK_TEXT);
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, DARK_BORDER);
        v.widgets.inactive.bg_fill   = DARK_BTN_BG;
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, DARK_TEXT);
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, DARK_BORDER);
        v.widgets.hovered.bg_fill    = DARK_BTN_HOVER;
        v.widgets.hovered.fg_stroke  = egui::Stroke::new(1.0, egui::Color32::WHITE);
        v.widgets.hovered.bg_stroke  = egui::Stroke::new(1.0, ACCENT);
        v.widgets.active.bg_fill     = ACCENT;
        v.widgets.active.fg_stroke   = egui::Stroke::new(1.0, egui::Color32::WHITE);
        v.widgets.active.bg_stroke   = egui::Stroke::new(1.0, ACCENT_HOVER);
        v.selection.bg_fill          = DARK_SEL;
        v.selection.stroke           = egui::Stroke::new(1.0, ACCENT);
        v.override_text_color        = Some(DARK_TEXT);
    } else {
        v.panel_fill           = LIGHT_BG;
        v.window_fill          = LIGHT_BG;
        v.faint_bg_color       = LIGHT_BG_RAISED;
        v.extreme_bg_color     = LIGHT_INPUT_BG;
        v.widgets.noninteractive.bg_fill   = LIGHT_BG;
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, LIGHT_TEXT);
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, LIGHT_BORDER);
        v.widgets.inactive.bg_fill   = LIGHT_BTN_BG;
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, LIGHT_TEXT);
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, LIGHT_BORDER);
        v.widgets.hovered.bg_fill    = LIGHT_BTN_HOVER;
        v.widgets.hovered.fg_stroke  = egui::Stroke::new(1.0, LIGHT_TEXT);
        v.widgets.hovered.bg_stroke  = egui::Stroke::new(1.0, ACCENT);
        v.widgets.active.bg_fill     = ACCENT;
        v.widgets.active.fg_stroke   = egui::Stroke::new(1.0, egui::Color32::WHITE);
        v.widgets.active.bg_stroke   = egui::Stroke::new(1.0, ACCENT_HOVER);
        v.selection.bg_fill          = LIGHT_SEL;
        v.selection.stroke           = egui::Stroke::new(1.0, ACCENT);
        v.override_text_color        = Some(LIGHT_TEXT);
    }

    v.window_rounding = egui::Rounding::same(6.0);
    v.window_shadow   = egui::epaint::Shadow::NONE;
    v.popup_shadow    = egui::epaint::Shadow::NONE;
    v.button_frame    = true;
    v
}

#[cfg(windows)]
fn read_windows_dark_mode() -> Option<bool> {
    use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER};
    use winapi::um::winnt::{KEY_READ, REG_DWORD};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let sub_key: Vec<u16> = OsStr::new(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
    ).encode_wide().chain(Some(0)).collect();
    let value_name: Vec<u16> = OsStr::new("AppsUseLightTheme")
        .encode_wide().chain(Some(0)).collect();

    unsafe {
        let mut hkey = std::mem::zeroed();
        let ret = RegOpenKeyExW(HKEY_CURRENT_USER, sub_key.as_ptr(), 0, KEY_READ, &mut hkey);
        if ret != 0 { return None; }
        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        let mut reg_type: u32 = 0;
        let ret = RegQueryValueExW(
            hkey, value_name.as_ptr(), std::ptr::null_mut(),
            &mut reg_type, &mut data as *mut u32 as *mut u8, &mut data_size,
        );
        winapi::um::winreg::RegCloseKey(hkey);
        if ret != 0 || reg_type != REG_DWORD { return None; }
        Some(data == 0)
    }
}
