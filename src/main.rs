#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::{ egui, WindowBuilderHook };
use egui::{ Key, Pos2, ScrollArea, Vec2, ViewportBuilder };
use rdev::{ listen, Event, EventType };
use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc, Mutex };
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tokio::{ runtime::Runtime, sync::broadcast };
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        BringWindowToTop,
        ShowWindow,
        SW_MINIMIZE,
        SW_RESTORE,
        SW_SHOWNORMAL,
    },
};
use winit::raw_window_handle::{ HasWindowHandle, RawWindowHandle, WindowHandle };
type SharedContext = Arc<Mutex<Option<egui::Context>>>;

async fn handle_event(
    event: Event,
    alt_pressed: &Arc<AtomicBool>,
    window_visible: &Arc<AtomicBool>,
    shared_ctx: SharedContext
) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if key == rdev::Key::Alt {
                alt_pressed.store(true, Ordering::SeqCst);
            } else if key == rdev::Key::CapsLock && alt_pressed.load(Ordering::SeqCst) {
                // Toggle window visibility
                let current = window_visible.load(Ordering::SeqCst);
                window_visible.store(!current, Ordering::SeqCst);
                println!("Window toggled: {}", !current);
                // Try to get the latest context
                if let Ok(ctx_guard) = shared_ctx.lock() {
                    if let Some(ctx) = ctx_guard.as_ref() {
                        println!("Got ctx, trying repaint");
                        ctx.request_repaint();
                    } else {
                        println!("No context available yet");
                    }
                } else {
                    println!("Failed to lock context mutex");
                }
            }
        }
        EventType::KeyRelease(key) => {
            if key == rdev::Key::Alt {
                alt_pressed.store(false, Ordering::SeqCst);
            }
        }
        _ => {}
    }
}

fn toggle_window(handle: RawWindowHandle, visible: bool) {
    match handle {
        RawWindowHandle::Win32(win32_handle) => unsafe {
            let hwnd = windows::Win32::Foundation::HWND(
                win32_handle.hwnd.get() as *mut std::ffi::c_void
            );

            if visible {
                // To show: restore from minimized or show normally
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
                let _ = BringWindowToTop(hwnd);
            } else {
                // To hide: minimize instead of SW_HIDE to avoid the issue
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
                // Remove from taskbar by changing extended style
                // SetWindowLongW(
                //     hwnd,
                //     GWL_EXSTYLE,
                //     GetWindowLongW(hwnd, GWL_EXSTYLE) | (WS_EX_TOOLWINDOW.0 as i32)
                // );
            }
        }
        _ => {
            println!("Unsupported platform or handle type");
        }
    }
}
fn main() -> eframe::Result {
    let title = "APP".to_string();
    let alt_pressed = Arc::new(AtomicBool::new(false));
    let window_visible = Arc::new(AtomicBool::new(true)); // Start with window visible
    let shared_ctx: SharedContext = Arc::new(Mutex::new(None));

    let ctrl_pressed_clone = Arc::clone(&alt_pressed);
    let window_visible_clone = Arc::clone(&window_visible);
    let shared_ctx_clone = Arc::clone(&shared_ctx);

    let runtime = Runtime::new().unwrap();

    runtime.spawn(async move {
        let listener = spawn_blocking(|| {
            if
                let Err(error) = listen(move |event| {
                    let ctrl_pressed_clone = Arc::clone(&ctrl_pressed_clone);
                    let window_visible_clone = Arc::clone(&window_visible_clone);
                    let shared_ctx_clone = Arc::clone(&shared_ctx_clone);
                    tokio::spawn(async move {
                        handle_event(
                            event,
                            &ctrl_pressed_clone,
                            &window_visible_clone,
                            shared_ctx_clone
                        ).await;
                    });
                })
            {
                println!("Error: {:?}", error);
            }
        });

        listener.await.unwrap();
    });
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            title: Some(title),
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
        Box::new(move |_cc| Ok(Box::new(Content::new(window_visible, shared_ctx))))
    );

    return r;
}

struct Content {
    text: String,
    window_visible: Arc<AtomicBool>,
    shared_ctx: SharedContext,
}

impl Content {
    fn new(window_visible: Arc<AtomicBool>, shared_ctx: SharedContext) -> Self {
        Self {
            text: String::new(),
            window_visible,
            shared_ctx,
        }
    }
}

// impl Default for Content {
//     fn default() -> Self {
//         Self {
//             text: String::new(),
//             window_visible: Arc::new(AtomicBool::new(true)),
//             ctx_tx,
//         }
//     }
// }
// TODO: move context to main and handle_event
// TODO: use context in handle_event to trigger update

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut ctx_guard) = self.shared_ctx.lock() {
            *ctx_guard = Some(ctx.clone());
        }
        let should_show = self.window_visible.load(Ordering::SeqCst);
        // println!("should show {}", should_show);
        // Only show UI if window should be visible
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Recent Apps");
            if ui.button("Clear").clicked() {
                self.text.clear();
            }
            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });

            if ctx.input(|i| i.key_released(Key::A)) {
                self.text.push_str("\nReleased");
            }
        });
        let window_handle = frame.window_handle().unwrap();
        toggle_window(window_handle.as_raw(), should_show);
    }
}
