use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc };

use egui::{ Key, Pos2, ScrollArea, Vec2, ViewportBuilder };
use winit::raw_window_handle::HasWindowHandle;

use crate::{
    recent_apps::{ get_open_windows },
    window_handler::{ focus_window, toggle_window },
    SharedContext,
};

pub static TITLE: &str = "RecentApps++";
pub fn render_ui(
    window_visible: Arc<AtomicBool>,
    shared_ctx: Arc<std::sync::Mutex<Option<egui::Context>>>
) -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            title: Some(TITLE.to_string()),
            app_id: Some("123123123".to_string()),
            active: Some(true),
            position: Some(Pos2::ZERO),
            inner_size: Some(Vec2 { x: 400.0, y: 400.0 }),
            min_inner_size: Some(Vec2 { x: 100.0, y: 100.0 }),
            max_inner_size: None,
            clamp_size_to_monitor_size: Some(true),
            fullscreen: None,
            maximized: None,
            resizable: Some(false),
            transparent: Some(true),
            decorations: Some(false),
            icon: None,
            visible: Some(true),
            fullsize_content_view: Some(true),
            movable_by_window_background: Some(false),
            title_shown: Some(false),
            titlebar_buttons_shown: Some(false),
            titlebar_shown: Some(false),
            has_shadow: Some(true),
            drag_and_drop: Some(false),
            taskbar: Some(false),
            close_button: Some(false),
            minimize_button: Some(false),
            maximize_button: Some(false),
            window_level: Some(egui::WindowLevel::AlwaysOnTop),
            mouse_passthrough: None,
            window_type: Some(egui::X11WindowType::Utility),
        },
        centered: true,
        ..Default::default()
    };
    // let options = eframe::NativeOptions::default();

    let r = eframe::run_native(
        "Keyboard events",
        native_options,
        Box::new(move |_cc| { Ok(Box::new(Content::new(window_visible, shared_ctx))) })
    );

    return r;
}

struct Content {
    selected_index: usize,
    search_text: String,
    window_visible: Arc<AtomicBool>,
    shared_ctx: SharedContext,
}

impl Content {
    fn new(window_visible: Arc<AtomicBool>, shared_ctx: SharedContext) -> Self {
        Self {
            selected_index: 0,
            search_text: String::new(),
            window_visible,
            shared_ctx,
        }
    }
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut ctx_guard) = self.shared_ctx.lock() {
            *ctx_guard = Some(ctx.clone());
        }
        let mut open_windows = get_open_windows();
        open_windows.sort();
        let filtered_windows: Vec<_> = open_windows
            .iter()
            .filter(|open_window| {
                open_window.title.to_lowercase().contains(&self.search_text.to_lowercase())
            })
            .filter(|open_window| open_window.title.to_lowercase() != TITLE.to_lowercase())
            .collect();
        let should_show = self.window_visible.load(Ordering::SeqCst);
        let window_handle = frame.window_handle().unwrap();
        let selected_index = &mut self.selected_index;
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                if *selected_index + 1 < filtered_windows.len() {
                    *selected_index += 1;
                }
            }
            if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                if *selected_index != 0 {
                    *selected_index -= 1;
                }
            }

            ui.heading(TITLE);
            let search_input = ui.text_edit_singleline(&mut self.search_text);
            if search_input.changed() {
                *selected_index = 0;
            }
            search_input.request_focus();
            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (idx, i) in filtered_windows.iter().enumerate() {
                        let is_selected = *selected_index == idx;

                        // Draw selectable label for keyboard navigation
                        let response = ui.selectable_label(is_selected, &i.title);
                        // Handle mouse click
                        if response.clicked() {
                            println!("{} clicked", i.title);
                            focus_window(i.hwnd);
                            self.window_visible.fetch_not(Ordering::SeqCst);
                            toggle_window(window_handle.as_raw(), false);
                        }

                        // Handle keyboard: Enter/Space to activate, Up/Down to move selection
                        if
                            is_selected &&
                            (ui.input(|i| i.key_pressed(Key::Enter)) ||
                                ui.input(|i| i.key_pressed(Key::Space)))
                        {
                            println!("{} activated by keyboard", i.title);
                            focus_window(i.hwnd);
                            self.window_visible.fetch_not(Ordering::SeqCst);
                            toggle_window(window_handle.as_raw(), false);
                        }
                    }
                });

            if ctx.input(|i| i.key_down(Key::Escape)) {
                self.window_visible.fetch_not(Ordering::SeqCst);
                toggle_window(window_handle.as_raw(), false);
            }
        });
        
        toggle_window(window_handle.as_raw(), should_show);
    }
}
