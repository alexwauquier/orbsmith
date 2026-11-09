//! The tray icon: reopens the window, lists running simulations, and quits.

use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::state::AppState;

const TRAY_ID: &str = "main";
const SHOW: &str = "show";
const STOP_ALL: &str = "stop-all";
const QUIT: &str = "quit";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(tooltip(0))
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW => show_main_window(app),
            STOP_ALL => {
                // Pause the queue first, or its next game would start.
                crate::queue::stop(app);
                app.state::<AppState>().simulations().stop_all();
                crate::simulations_changed(app);
            }
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Rebuilds the menu and tooltip from the running simulations.
pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let running = app.state::<AppState>().simulations().list().len();
    let result = menu(app)
        .and_then(|menu| tray.set_menu(Some(menu)))
        .and_then(|()| tray.set_tooltip(Some(tooltip(running))));
    if let Err(err) = result {
        log::warn!("failed to update the tray: {err}");
    }
}

pub fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let result = window
        .show()
        .and_then(|()| window.unminimize())
        .and_then(|()| window.set_focus());
    if let Err(err) = result {
        log::warn!("failed to show the window: {err}");
    }
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let running = app.state::<AppState>().simulations().list();
    let mut builder = MenuBuilder::new(app)
        .text(SHOW, "Show Orbsmith")
        .separator();
    if running.is_empty() {
        builder = builder.item(&MenuItem::with_id(
            app,
            "nothing-running",
            "Nothing is running",
            false,
            None::<&str>,
        )?);
    } else {
        for sim in &running {
            builder = builder.item(&MenuItem::with_id(
                app,
                format!("running:{}", sim.app_id),
                &sim.app_name,
                false,
                None::<&str>,
            )?);
        }
        builder = builder.text(STOP_ALL, "Stop all");
    }
    let remaining = {
        let state = app.state::<AppState>();
        let queue = state.queue();
        queue.is_running().then(|| queue.remaining())
    };
    if let Some(remaining) = remaining {
        let label = match remaining {
            1 => "Queue: 1 left".to_owned(),
            n => format!("Queue: {n} left"),
        };
        builder = builder.separator().item(&MenuItem::with_id(
            app,
            "queue",
            label,
            false,
            None::<&str>,
        )?);
    }
    builder.separator().text(QUIT, "Quit Orbsmith").build()
}

fn tooltip(running: usize) -> String {
    match running {
        0 => "Orbsmith".to_owned(),
        1 => "Orbsmith: 1 game running".to_owned(),
        n => format!("Orbsmith: {n} games running"),
    }
}
