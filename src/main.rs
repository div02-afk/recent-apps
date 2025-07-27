#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
mod recent_apps;
mod ui;
mod window_handler;

use crate::ui::render_ui;

use eframe::egui;
use rdev::{ listen, Event, EventType };
use std::sync::{ atomic::{ AtomicBool, Ordering }, Arc, Mutex };
use tokio::task::spawn_blocking;
use tokio::runtime::Runtime;

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

fn main() -> eframe::Result {
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
    render_ui(window_visible, shared_ctx)
}
