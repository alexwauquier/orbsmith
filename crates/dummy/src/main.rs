//! Stand-in process. Orbsmith copies this binary under a game's executable name
//! and runs it so that game detection sees the game as running.
//!
//! It does nothing but stay alive, with a small window on Windows, and exits as
//! soon as the Orbsmith process that launched it is gone.

#![cfg_attr(windows, windows_subsystem = "windows")]

/// Must match `orbsmith_core::simulation::PARENT_PID_ENV`.
const PARENT_PID_ENV: &str = "ORBSMITH_PARENT_PID";
/// Must match `orbsmith_core::simulation::TITLE_ENV`.
const TITLE_ENV: &str = "ORBSMITH_TITLE";

fn main() {
    if let Some(pid) = std::env::var(PARENT_PID_ENV)
        .ok()
        .and_then(|pid| pid.parse::<u32>().ok())
    {
        platform::exit_with_parent(pid);
    }

    let title = std::env::var(TITLE_ENV).unwrap_or_else(|_| "Orbsmith".to_owned());
    platform::run(&title);
}

#[cfg(windows)]
mod platform {
    use std::ptr::null;

    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::{
        INFINITE, OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, IDC_ARROW,
        LoadCursorW, MSG, PostQuitMessage, RegisterClassW, SW_SHOWMINNOACTIVE, ShowWindow,
        TranslateMessage, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    };

    /// Exits the process once `pid` terminates. If it is already gone, exits
    /// right away.
    pub fn exit_with_parent(pid: u32) {
        // SAFETY: plain Win32 call; a null handle is checked below.
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        if handle.is_null() {
            std::process::exit(0);
        }
        // Raw handles are not Send; pass the value as an integer.
        let handle = handle as usize;
        std::thread::spawn(move || {
            // SAFETY: the handle stays open for the life of the process.
            unsafe { WaitForSingleObject(handle as _, INFINITE) };
            std::process::exit(0);
        });
    }

    /// Shows a minimized, empty window and runs its message loop until the
    /// window is closed.
    pub fn run(title: &str) {
        let class_name = wide("OrbsmithDummy");
        let title = wide(title);

        // SAFETY: standard Win32 window setup. All pointers passed point to
        // live, NUL-terminated buffers owned by this function.
        unsafe {
            let instance = GetModuleHandleW(null());
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: class_name.as_ptr(),
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as usize as HBRUSH,
                ..std::mem::zeroed()
            };
            RegisterClassW(&class);

            let window = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                320,
                120,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                null(),
            );
            if window.is_null() {
                return;
            }
            ShowWindow(window, SW_SHOWMINNOACTIVE);

            let mut message: MSG = std::mem::zeroed();
            while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: forwarding the arguments Windows passed in.
        unsafe {
            if message == WM_DESTROY {
                PostQuitMessage(0);
                return 0;
            }
            DefWindowProcW(window, message, wparam, lparam)
        }
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(unix)]
mod platform {
    use std::time::Duration;

    /// Exits the process once `pid` is no longer its parent. When the parent
    /// dies, the process is reparented and its parent PID changes.
    pub fn exit_with_parent(pid: u32) {
        if std::os::unix::process::parent_id() != pid {
            std::process::exit(0);
        }
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                if std::os::unix::process::parent_id() != pid {
                    std::process::exit(0);
                }
            }
        });
    }

    pub fn run(_title: &str) {
        loop {
            std::thread::park();
        }
    }
}
