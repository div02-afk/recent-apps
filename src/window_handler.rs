use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{ EnableWindow, SetFocus },
        WindowsAndMessaging::{
            BringWindowToTop,
            CloseWindow,
            ShowWindow,
            SwitchToThisWindow,
            SW_MINIMIZE,
            SW_RESTORE,
            SW_SHOWNORMAL,
        },
    },
};
use winit::raw_window_handle::RawWindowHandle;

pub fn toggle_window(handle: RawWindowHandle, visible: bool) {
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
                let _ = focus_window(hwnd);
            } else {
                // To hide: minimize instead of SW_HIDE to avoid the issue
                // let _ = ShowWindow(hwnd, SW_MINIMIZE);
                let _ = CloseWindow(hwnd);
                // Remove from taskbar by changing extended style
                std::process::exit(0);
            }
        }
        _ => {
            println!("Unsupported platform or handle type");
        }
    }
}

pub fn focus_window(hwnd: HWND) {
    unsafe {
        let _ = SwitchToThisWindow(hwnd, true);
        let _ = SetFocus(Some(hwnd));
        let _ = EnableWindow(hwnd, true);
    }
}
