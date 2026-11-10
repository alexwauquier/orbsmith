//! The tray icon: reopens the window, starts favorites, lists running
//! simulations, and quits.

use std::collections::HashSet;
use std::time::Duration;

use orbsmith_core::library::Launch;
use orbsmith_core::{Library, Os};
use tauri::menu::{Menu, MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::state::AppState;
use crate::{commands, notify};

const TRAY_ID: &str = "main";
const SHOW: &str = "show";
const STOP_ALL: &str = "stop-all";
const QUIT: &str = "quit";
/// Prefix of the menu IDs that start a favorite, followed by its app ID.
const FAVORITE: &str = "favorite:";
/// Favorites listed in the tray; the rest are in the window.
const FAVORITES_LIMIT: usize = 10;

/// Emitted with the app ID when a game was started from the tray, so the
/// frontend moves it to the front of the recent games.
const TRAY_STARTED_EVENT: &str = "tray-started";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(tooltip(0))
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW => show_main_window(app),
            STOP_ALL => {
                // Pause the queue first, or its next game would start. Its
                // games keep their time left for when it is resumed.
                crate::queue::pause(app);
                app.state::<AppState>().simulations().stop_all();
                crate::simulations_changed(app);
            }
            QUIT => app.exit(0),
            id => {
                if let Some(app_id) = id.strip_prefix(FAVORITE) {
                    start_favorite(app, app_id.to_owned());
                }
            }
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

/// The favorites to list, if the game list is loaded. It is never
/// downloaded for the menu: the window loads it at startup, then refreshes
/// the tray.
fn favorites(state: &AppState) -> Vec<Launch> {
    let Some(catalog) = state
        .catalog
        .try_lock()
        .ok()
        .and_then(|slot| slot.as_ref().cloned())
    else {
        return Vec::new();
    };
    match Library::load(&state.layout.library_file()) {
        Ok(Some(library)) => library.favorite_launches(&catalog, Os::CURRENT, FAVORITES_LIMIT),
        Ok(None) => Vec::new(),
        Err(err) => {
            log::warn!("failed to read the library for the tray: {err}");
            Vec::new()
        }
    }
}

/// Starts a favorite the way it was last started, in the background, and
/// reports a failure as a notification since the window may be hidden.
fn start_favorite(app: &AppHandle, app_id: String) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        match try_start_favorite(&state, &app_id).await {
            Ok(()) => {
                crate::simulations_changed(&app);
                if let Err(err) = app.emit(TRAY_STARTED_EVENT, &app_id) {
                    log::warn!("failed to emit {TRAY_STARTED_EVENT}: {err}");
                }
            }
            Err((app_name, message)) => {
                log::warn!("failed to start {app_name} from the tray: {message}");
                notify::start_failed(&app, &app_name, &message);
            }
        }
    });
}

/// On failure, returns the game's name, if known, and why.
async fn try_start_favorite(state: &AppState, app_id: &str) -> Result<(), (String, String)> {
    let unnamed = |message: String| ("the game".to_owned(), message);
    let catalog = commands::catalog(state, false)
        .await
        .map_err(|err| unnamed(err.message().to_owned()))?;
    let library = Library::load(&state.layout.library_file())
        .map_err(|err| unnamed(err.to_string()))?
        .unwrap_or_default();
    let unknown = || unnamed(format!("{app_id} is no longer a favorite in the game list"));
    let launch = library
        .favorite_launches(&catalog, Os::CURRENT, usize::MAX)
        .into_iter()
        .find(|launch| launch.app_id == app_id)
        .ok_or_else(unknown)?;
    let detectable = catalog.get(app_id).ok_or_else(unknown)?;
    let started = state.simulations().start(
        detectable,
        Os::CURRENT,
        launch.executable.as_deref(),
        launch.duration,
    );
    started
        .map(|_| ())
        .map_err(|err| (launch.app_name, err.to_string()))
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let running = app.state::<AppState>().simulations().list();
    let mut builder = MenuBuilder::new(app)
        .text(SHOW, "Show Orbsmith")
        .separator();
    let favorites = favorites(&app.state::<AppState>());
    if !favorites.is_empty() {
        let running_ids: HashSet<&str> = running.iter().map(|sim| sim.app_id.as_str()).collect();
        let mut submenu = SubmenuBuilder::new(app, "Start a favorite");
        for launch in &favorites {
            submenu = submenu.item(&MenuItem::with_id(
                app,
                format!("{FAVORITE}{}", launch.app_id),
                favorite_label(launch),
                !running_ids.contains(launch.app_id.as_str()),
                None::<&str>,
            )?);
        }
        builder = builder.item(&submenu.build()?).separator();
    }
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
                menu_text(&sim.app_name),
                false,
                None::<&str>,
            )?);
        }
        builder = builder.text(STOP_ALL, "Stop all");
    }
    let (remaining, paused) = {
        let state = app.state::<AppState>();
        let queue = state.queue();
        let shown = queue.is_running() || queue.is_paused();
        (shown.then(|| queue.remaining()), queue.is_paused())
    };
    if let Some(remaining) = remaining {
        let title = if paused { "Queue paused" } else { "Queue" };
        let label = format!("{title}: {remaining} left");
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

/// `Game · 15 min`, `Game · 1 h 30 min`, or `Game · until stopped`.
fn favorite_label(launch: &Launch) -> String {
    let duration = launch
        .duration
        .map_or_else(|| "until stopped".to_owned(), format_duration);
    format!("{} · {duration}", menu_text(&launch.app_name))
}

/// A game name as menu text: a single `&` marks a keyboard mnemonic.
fn menu_text(name: &str) -> String {
    name.replace('&', "&&")
}

/// Mirrors `formatDuration` in `src/lib/time.ts`.
fn format_duration(duration: Duration) -> String {
    let minutes = duration.as_secs() / 60;
    match (minutes / 60, minutes % 60) {
        (0, rest) => format!("{rest} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, rest) => format!("{hours} h {rest} min"),
    }
}

fn tooltip(running: usize) -> String {
    match running {
        0 => "Orbsmith".to_owned(),
        1 => "Orbsmith: 1 game running".to_owned(),
        n => format!("Orbsmith: {n} games running"),
    }
}
