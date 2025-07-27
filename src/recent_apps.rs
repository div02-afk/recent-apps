use windows::core::{Error, GUID, PWSTR};
use windows::Win32::{
    Foundation::{self, HWND, LPARAM},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED},
    UI::{
        Shell::IVirtualDesktopManager,
        WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowLongW, GetWindowRect, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GWL_STYLE, WS_EX_TOOLWINDOW,
            WS_OVERLAPPEDWINDOW, WS_POPUP,
        },
    },
};
use windows_result::BOOL;

#[derive(Debug)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
}

impl PartialEq for WindowInfo {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}
impl Eq for WindowInfo {}

impl PartialOrd for WindowInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.title.cmp(&other.title))
    }
}

impl Ord for WindowInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let open_windows = (lparam.0 as *mut Vec<WindowInfo>).as_mut();
    if let Some(open_windows) = open_windows {
        let mut buffer = [0u16; 512]; // Buffer to store window title
                                      // Get window title
        let length = GetWindowTextW(hwnd, &mut buffer);
        let title = String::from_utf16_lossy(&buffer[..length as usize]);

        // Get window class name
        let class_length = GetClassNameW(hwnd, &mut buffer);
        let class_name = String::from_utf16_lossy(&buffer[..class_length as usize]);

        // Get Process ID of the window
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Check if window is visible
        if !title.is_empty() && IsWindowVisible(hwnd).as_bool() {
            let style = GetWindowLongW(hwnd, GWL_STYLE);

            // Check if it has typical GUI window styles
            let is_gui =
                (style & (WS_OVERLAPPEDWINDOW.0 as i32)) != 0 || (style & (WS_POPUP.0 as i32)) != 0;

            // Check if it's not a tool window or other special windows
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let is_not_tool = (ex_style & (WS_EX_TOOLWINDOW.0 as i32)) == 0;
            let mut rect = Foundation::RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);
            let has_size = rect.right - rect.left > 0 && rect.bottom - rect.top > 0;

            // Check if window is cloaked (hidden but maintained by the system)
            let mut cloaked: BOOL = BOOL(0);
            let cloaked_check = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut BOOL as *mut _,
                std::mem::size_of::<BOOL>() as u32,
            );
            //use this if only current desktop's apps needed
            let not_cloaked = cloaked_check.is_err() || !cloaked.as_bool();
            // let path = get_executable_path_from_pid(process_id).unwrap_or_default();
            if is_gui && is_not_tool && has_size && not_cloaked {
                open_windows.push(WindowInfo {
                    hwnd,
                    title,
                    class_name,
                    process_id,
                });
                // println!(
                //     "GUI Window - HWND: {:?}, Title: \"{}\", Class: \"{}\", PID: {}",
                //     hwnd,
                //     title,
                //     class_name,
                //     process_id
                // );
            }
        }
    }
    BOOL(1) // Continue enumeration
}

pub fn get_open_windows(desktop_manager: &IVirtualDesktopManager) -> Vec<WindowInfo> {
    unsafe {
        let mut open_windows: Vec<WindowInfo> = Vec::new();
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut open_windows as *mut _ as isize),
        );

        // println!("{:?}", &open_windows);
        return open_windows;
    }
}

pub fn create_virtual_desktop_manager() -> Result<IVirtualDesktopManager, Error> {
    unsafe {
        let res = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if res.is_err() {
            println!("Failed");
            ();
        }

        let clsid = GUID::from_values(
            0xaa509086,
            0x5ca9,
            0x4c25,
            [0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a],
        );

        // Create an instance of IVirtualDesktopManager
        let desktop_manager_result: windows::core::Result<IVirtualDesktopManager> =
            CoCreateInstance(
                &clsid,     // CLSID of IVirtualDesktopManager
                None,       // No aggregation
                CLSCTX_ALL, // Create in all COM contexts
            );
        match desktop_manager_result {
            Ok(_) => {
                return Ok(desktop_manager_result.unwrap());
            }
            Err(e) => {
                eprintln!("CoCreateInstance failed: {:?}", e);
                return Err(e);
            }
        }
    }
}
