// Prunner — Portable Windows Run
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod history;
mod executor;
mod theme;

use eframe::egui;
use history::History;
use theme::{Colors, ACCENT, ERROR};

const APP_TITLE:     &str = "Prunner";
const WINDOW_WIDTH:  f32  = 430.0;
const WINDOW_HEIGHT: f32  = 158.0;

// ComboBox geometry
const COMBO_H:       f32  = 23.0;  // inner input height (Win-style compact)
const ARROW_W:       f32  = 18.0;  // width of the embedded arrow zone

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false)
            .with_always_on_top()
            .with_icon(load_icon()),
        ..Default::default()
    };
    eframe::run_native(APP_TITLE, options, Box::new(|cc| Box::new(PrunnerApp::new(cc))))
}

fn load_icon() -> std::sync::Arc<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../assets/run.ico");
    if let Ok(ico) = ico::IconDir::read(std::io::Cursor::new(bytes)) {
        if let Some(e) = ico.entries().first() {
            if let Ok(img) = e.decode() {
                return std::sync::Arc::new(egui::IconData {
                    rgba: img.rgba_data().to_vec(),
                    width: img.width(),
                    height: img.height(),
                });
            }
        }
    }
    std::sync::Arc::new(egui::IconData { rgba: vec![], width: 0, height: 0 })
}

#[derive(PartialEq)]
enum ConfirmAction { None, ClearHistory }

struct PrunnerApp {
    input: String,
    history: History,
    history_index: Option<usize>,
    error_message: Option<String>,
    confirm_action: ConfirmAction,
    first_frame: bool,
    positioned: bool,
    dropdown_open: bool,
    // Full rect of the hand-drawn combo box (used to anchor popup)
    combo_rect: egui::Rect,
}

impl PrunnerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.tessellation_options_mut(|o| {
            o.feathering = false;
            o.feathering_size_in_pixels = 0.0;
        });
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing   = egui::vec2(6.0, 4.0);
            s.spacing.button_padding = egui::vec2(12.0, 5.0);
        });
        Self {
            input: String::new(),
            history: History::load(),
            history_index: None,
            error_message: None,
            confirm_action: ConfirmAction::None,
            first_frame: true,
            positioned: false,
            dropdown_open: false,
            combo_rect: egui::Rect::NOTHING,
        }
    }

    fn run_command(&mut self, as_admin: bool, ctx: &egui::Context) {
        let raw = self.input.trim().to_string();
        if raw.is_empty() { return; }
        match executor::execute(&executor::expand_env_vars(&raw), as_admin) {
            Ok(()) => {
                self.history.push(raw);
                self.history.save();
                self.history_index = None;
                self.dropdown_open = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => { self.error_message = Some(e); }
        }
    }

    fn browse(&mut self) {
        #[cfg(windows)] {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            let mut file_name = [0u16; 32768];
            let mut ofn: winapi::um::commdlg::OPENFILENAMEW = unsafe { std::mem::zeroed() };
            ofn.lStructSize = std::mem::size_of::<winapi::um::commdlg::OPENFILENAMEW>() as u32;
            ofn.lpstrFile   = file_name.as_mut_ptr();
            ofn.nMaxFile    = file_name.len() as u32;
            let filter: Vec<u16> = OsStr::new("All Files\0*.*\0\0").encode_wide().collect();
            ofn.lpstrFilter = filter.as_ptr();
            ofn.Flags = winapi::um::commdlg::OFN_FILEMUSTEXIST | winapi::um::commdlg::OFN_PATHMUSTEXIST;
            if unsafe { winapi::um::commdlg::GetOpenFileNameW(&mut ofn) } != 0 {
                let end = file_name.iter().position(|&c| c == 0).unwrap_or(0);
                self.input = String::from_utf16_lossy(&file_name[..end]);
                self.error_message = None;
            }
        }
        #[cfg(not(windows))] {
            self.error_message = Some("Browse is only supported on Windows.".to_string());
        }
    }

    fn history_up(&mut self) {
        if self.history.entries().is_empty() { return; }
        let idx = match self.history_index {
            None    => 0,
            Some(i) => (i + 1).min(self.history.entries().len() - 1),
        };
        self.history_index = Some(idx);
        self.input = self.history.entries()[idx].clone();
        self.error_message = None;
    }

    fn history_down(&mut self) {
        match self.history_index {
            None    => {}
            Some(0) => { self.history_index = None; self.input.clear(); }
            Some(i) => {
                let idx = i - 1;
                self.history_index = Some(idx);
                self.input = self.history.entries()[idx].clone();
            }
        }
        self.error_message = None;
    }

    /// Draw a Windows-style ComboBox: single bordered rect, arrow drawn
    /// inside the right edge. Returns (text_resp, arrow_clicked, full_rect).
    fn combo_box<'a>(
        ui: &mut egui::Ui,
        input: &'a mut String,
        c: &Colors,
        focused: bool,
        dropdown_open: bool,
    ) -> (egui::Response, bool, egui::Rect) {
        let avail_w = ui.available_width();
        let outer_h = COMBO_H + 6.0; // include 3px top+bottom border/pad

        // Allocate the full combo rectangle
        let (outer_rect, _outer_resp) = ui.allocate_exact_size(
            egui::vec2(avail_w, outer_h),
            egui::Sense::hover(),
        );

        let painter = ui.painter();

        // Border colour: blue when focused, standard otherwise
        let border_col = if focused {
            ACCENT
        } else if c.dark {
            egui::Color32::from_rgb(130, 130, 145)
        } else {
            egui::Color32::from_rgb(120, 120, 130)
        };

        // Background
        painter.rect_filled(outer_rect, 2.0, c.input_bg);
        // Border
        painter.rect_stroke(outer_rect, 2.0, egui::Stroke::new(1.0, border_col));

        // Arrow divider line (vertical, inside right edge)
        let divider_x = outer_rect.right() - ARROW_W;
        painter.vline(
            divider_x,
            outer_rect.y_range(),
            egui::Stroke::new(1.0, border_col),
        );

        // Arrow glyph
        let arrow_center = egui::pos2(
            divider_x + ARROW_W / 2.0,
            outer_rect.center().y,
        );
        let arrow_ch = if dropdown_open { "▲" } else { "▼" };
        painter.text(
            arrow_center,
            egui::Align2::CENTER_CENTER,
            arrow_ch,
            egui::FontId::proportional(8.0),
            c.text,
        );

        // Arrow click zone
        let arrow_rect = egui::Rect::from_min_size(
            egui::pos2(divider_x, outer_rect.top()),
            egui::vec2(ARROW_W, outer_h),
        );
        let arrow_resp = ui.interact(arrow_rect, egui::Id::new("combo_arrow"), egui::Sense::click());
        let arrow_clicked = arrow_resp.clicked();

        // Text edit — placed inside the left portion, inset from borders
        let text_rect = egui::Rect::from_min_size(
            outer_rect.min + egui::vec2(4.0, 2.0),
            egui::vec2(divider_x - outer_rect.left() - 6.0, COMBO_H),
        );

        let mut child_ui = ui.child_ui(text_rect, egui::Layout::left_to_right(egui::Align::Center));
        let te = egui::TextEdit::singleline(input)
            .frame(false)                           // no border — we draw our own
            .hint_text("Enter command, path, or URL…")
            .font(egui::FontId::proportional(13.0))
            .text_color(c.text)
            .desired_width(text_rect.width());
        let text_resp = child_ui.add(te);

        (text_resp, arrow_clicked, outer_rect)
    }
}

impl eframe::App for PrunnerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.positioned {
            self.positioned = true;
            let screen = ctx.screen_rect();
            let margin = 48.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                margin,
                (screen.height() - WINDOW_HEIGHT - margin - 40.0).max(0.0),
            )));
        }

        theme::apply(ctx);
        let c = Colors::from_ctx(ctx);

        let esc   = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let up    = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let down  = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let run   = ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.is_none());
        let admin = ctx.input(|i| {
            i.key_pressed(egui::Key::Enter)
                && (i.modifiers.ctrl || i.modifiers.alt || i.modifiers.shift)
        });

        if esc {
            if self.dropdown_open { self.dropdown_open = false; }
            else { ctx.send_viewport_cmd(egui::ViewportCommand::Close); return; }
        }
        if up   { self.history_up();   self.dropdown_open = true; }
        if down { self.history_down(); self.dropdown_open = true; }

        let mut do_run   = run   && self.confirm_action == ConfirmAction::None;
        let do_admin = admin && self.confirm_action == ConfirmAction::None;

        // ── Main window panel ─────────────────────────────────────────────
        let panel_frame = egui::Frame::none()
            .fill(c.bg)
            .inner_margin(egui::Margin { left: 12.0, right: 12.0, top: 12.0, bottom: 10.0 });

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {

            // ── Subtitle row: text left, ℹ right ──────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Alternative Portable Windows Runner")
                        .size(13.0)
                        .color(c.text),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let info_resp = ui.add(
                        egui::Button::new(egui::RichText::new("ℹ").size(12.0).color(c.text_dim))
                            .min_size(egui::vec2(20.0, 20.0))
                            .frame(false),
                    );
                    if info_resp.hovered() {
                        egui::show_tooltip_at_pointer(ctx, egui::Id::new("admin_tip"), |ui| {
                            ui.label(
                                egui::RichText::new("Ctrl+Enter  —  Run as Administrator")
                                    .size(12.0),
                            );
                        });
                    }
                });
            });

            ui.add_space(8.0);

            // Horizontal rule
            let sep_rect = ui.available_rect_before_wrap();
            ui.painter().hline(
                sep_rect.x_range(),
                sep_rect.top(),
                egui::Stroke::new(1.0, c.sep),
            );
            ui.add_space(10.0);

            // ── "Open:" label + ComboBox ──────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Open:")
                        .size(13.0)
                        .color(c.text),
                );
                ui.add_space(4.0);

                let focused = ctx.memory(|m| m.focused().is_some());
                let (text_resp, arrow_clicked, combo_rect) =
                    PrunnerApp::combo_box(ui, &mut self.input, &c, focused, self.dropdown_open);

                self.combo_rect = combo_rect;

                if self.first_frame { text_resp.request_focus(); self.first_frame = false; }
                if text_resp.changed() { self.error_message = None; self.history_index = None; }
                if arrow_clicked {
                    self.dropdown_open = !self.dropdown_open;
                    self.confirm_action = ConfirmAction::None;
                }
            });

            // ── Error message ─────────────────────────────────────────────
            if let Some(ref err) = self.error_message.clone() {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(52.0);
                    ui.label(
                        egui::RichText::new(format!("⚠  {}", err))
                            .size(11.5)
                            .color(ERROR),
                    );
                });
            }

            ui.add_space(10.0);

            // ── Buttons: OK · Cancel · Browse… right-aligned ─────────────
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Browse
                if ui.add(
                    egui::Button::new(egui::RichText::new("Browse…").size(13.0).color(c.text))
                        .min_size(egui::vec2(75.0, 26.0))
                        .fill(c.btn_bg)
                        .stroke(egui::Stroke::new(1.0, c.border)),
                ).clicked() { self.browse(); }

                // Cancel
                if ui.add(
                    egui::Button::new(egui::RichText::new("Cancel").size(13.0).color(c.text))
                        .min_size(egui::vec2(70.0, 26.0))
                        .fill(c.btn_bg)
                        .stroke(egui::Stroke::new(1.0, c.border)),
                ).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }

                // OK — accent when input has text, greyed out when empty
                let has_input = !self.input.trim().is_empty();
                let ok_fill   = if has_input { ACCENT } else { c.btn_bg };
                let ok_text   = egui::RichText::new("OK")
                    .size(13.0)
                    .strong()
                    .color(if has_input { egui::Color32::WHITE } else { c.text_dim });
                let ok_btn = ui.add_enabled(
                    has_input,
                    egui::Button::new(ok_text)
                        .min_size(egui::vec2(60.0, 26.0))
                        .fill(ok_fill)
                        .stroke(egui::Stroke::new(1.0, c.border)),
                );
                if ok_btn.clicked() { do_run = true; }
            });
        });

        // ── History dropdown popup ────────────────────────────────────────
        if self.dropdown_open && self.combo_rect != egui::Rect::NOTHING {
            let popup_x = self.combo_rect.left();
            let popup_y = self.combo_rect.bottom() + 1.0;
            let popup_w = self.combo_rect.width();

            let area_resp = egui::Area::new(egui::Id::new("history_dd"))
                .fixed_pos(egui::pos2(popup_x, popup_y))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(c.bg_raised)
                        .stroke(egui::Stroke::new(1.0, c.border))
                        .rounding(egui::Rounding { nw: 0.0, ne: 0.0, sw: 3.0, se: 3.0 })
                        .inner_margin(egui::Margin::symmetric(0.0, 3.0))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::vec2(0.0, 4.0),
                            blur: 10.0,
                            spread: 0.0,
                            color: egui::Color32::from_black_alpha(60),
                        })
                        .show(ui, |ui| {
                            ui.set_width(popup_w);
                            let entries = self.history.entries().to_vec();

                            if entries.is_empty() {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new("No history yet.")
                                            .size(12.5)
                                            .color(c.text_dim),
                                    );
                                });
                                ui.add_space(4.0);
                            } else {
                                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    ui.set_width(popup_w);
                                    for (i, entry) in entries.iter().enumerate() {
                                        let selected = self.history_index == Some(i);
                                        let (item_rect, item_resp) = ui.allocate_exact_size(
                                            egui::vec2(popup_w, 22.0),
                                            egui::Sense::click(),
                                        );
                                        let fill = if selected {
                                            c.sel
                                        } else if item_resp.hovered() {
                                            c.hover
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };
                                        ui.painter().rect_filled(item_rect, 0.0, fill);
                                        // Text colour: white on selection for dark, dark-on-blue for light
                                        let txt_col = if selected {
                                            if c.dark { egui::Color32::WHITE }
                                            else { egui::Color32::from_rgb(0, 0, 0) }
                                        } else {
                                            c.text
                                        };
                                        ui.painter().text(
                                            egui::pos2(item_rect.left() + 10.0, item_rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            entry.as_str(),
                                            egui::FontId::proportional(13.0),
                                            txt_col,
                                        );
                                        if item_resp.clicked() {
                                            self.input = entry.clone();
                                            self.history_index = Some(i);
                                            self.error_message = None;
                                            self.dropdown_open = false;
                                        }
                                    }
                                });

                                // Divider + clear footer
                                let cur_y = ui.cursor().top();
                                ui.painter().hline(
                                    egui::Rangef::new(popup_x, popup_x + popup_w),
                                    cur_y,
                                    egui::Stroke::new(1.0, c.sep),
                                );
                                ui.add_space(2.0);

                                if self.confirm_action == ConfirmAction::ClearHistory {
                                    ui.horizontal(|ui| {
                                        ui.add_space(10.0);
                                        ui.label(
                                            egui::RichText::new("Clear all history?")
                                                .size(12.0)
                                                .color(ERROR),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            if ui.add(egui::Button::new(
                                                egui::RichText::new("Cancel").size(11.5),
                                            ).small()).clicked() {
                                                self.confirm_action = ConfirmAction::None;
                                            }
                                            ui.add_space(4.0);
                                            if ui.add(egui::Button::new(
                                                egui::RichText::new("Yes, clear")
                                                    .size(11.5)
                                                    .color(egui::Color32::WHITE),
                                            ).fill(ERROR).stroke(egui::Stroke::NONE).small()).clicked() {
                                                self.history.clear();
                                                self.history.save();
                                                self.history_index = None;
                                                self.confirm_action = ConfirmAction::None;
                                                self.dropdown_open = false;
                                            }
                                        });
                                    });
                                    ui.add_space(2.0);
                                } else {
                                    let (row_rect, row_resp) = ui.allocate_exact_size(
                                        egui::vec2(popup_w, 24.0),
                                        egui::Sense::click(),
                                    );
                                    let row_fill = if row_resp.hovered() { c.hover } else { egui::Color32::TRANSPARENT };
                                    ui.painter().rect_filled(row_rect, 0.0, row_fill);
                                    ui.painter().text(
                                        egui::pos2(row_rect.left() + 10.0, row_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        "— Clear history",
                                        egui::FontId::proportional(12.5),
                                        ERROR,
                                    );
                                    if row_resp.clicked() {
                                        self.confirm_action = ConfirmAction::ClearHistory;
                                    }
                                }
                            }
                        });
                });

            // Close on outside click
            let popup_bounds = area_resp.response.rect;
            if ctx.input(|i| i.pointer.any_click()) {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if !popup_bounds.contains(pos) && !self.combo_rect.contains(pos) {
                        self.dropdown_open = false;
                        self.confirm_action = ConfirmAction::None;
                    }
                }
            }
        }

        if do_run   { self.run_command(false, ctx); }
        if do_admin { self.run_command(true,  ctx); }
    }
}
