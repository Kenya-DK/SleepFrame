//! Thin Windows interop layer: locating the Warframe process/window and
//! manipulating window focus. Ported from `Helper.cs`.

use windows_sys::Win32::Foundation::{
    CloseHandle, HWND, INVALID_HANDLE_VALUE, LPARAM, POINT, RECT,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    BlockInput, GetAsyncKeyState, SetFocus, VK_LBUTTON,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowTextW,
    GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
    ShowWindowAsync, SW_RESTORE,
};

/// Executable name (without extension) of the game process.
pub const WARFRAME_EXE: &str = "Warframe.x64";

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

/// Finds the process id of the first process whose executable name starts with
/// `name` (case-insensitive).
pub fn find_process_pid(name: &str) -> Option<u32> {
    let needle = name.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return None;
    }

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = None;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let name = wide_to_string(&entry.szExeFile);
                if name.to_ascii_lowercase().starts_with(&needle) {
                    found = Some(entry.th32ProcessID);
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        found
    }
}

/// Finds the executable name for a process id.
pub fn process_name_by_pid(pid: u32) -> Option<String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = None;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                if entry.th32ProcessID == pid {
                    found = Some(wide_to_string(&entry.szExeFile));
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        found
    }
}

/// Finds the process id of a running `Warframe.x64` process, if any.
pub fn find_warframe_pid() -> Option<u32> {
    find_process_pid(WARFRAME_EXE)
}

/// Returns a window's title text.
pub fn window_title(hwnd: HWND) -> String {
    if hwnd.is_null() {
        return String::new();
    }
    unsafe {
        let mut buffer = [0u16; 512];
        let length = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if length <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..length as usize])
    }
}

/// Returns the process id that owns a window.
pub fn pid_of_window(hwnd: HWND) -> u32 {
    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    pid
}

struct TitleSearch {
    needle: String,
    found: HWND,
}

unsafe extern "system" fn enum_title_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let search = &mut *(lparam as *mut TitleSearch);
    if IsWindowVisible(hwnd) != 0 {
        let title = window_title(hwnd);
        if title.to_lowercase().contains(&search.needle) {
            search.found = hwnd;
            return 0;
        }
    }
    1
}

/// Finds the first visible window whose title contains `needle` (case-insensitive).
pub fn find_window_by_title(needle: &str) -> Option<HWND> {
    let mut search = TitleSearch {
        needle: needle.trim().to_lowercase(),
        found: std::ptr::null_mut(),
    };
    unsafe {
        EnumWindows(
            Some(enum_title_proc),
            &mut search as *mut TitleSearch as LPARAM,
        );
    }
    if search.found.is_null() {
        None
    } else {
        Some(search.found)
    }
}

#[derive(Default, Clone, Copy)]
struct WindowSearch {
    pid: u32,
    titled: HWND,
    visible: HWND,
    titled_set: bool,
    visible_set: bool,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let search = &mut *(lparam as *mut WindowSearch);
    let mut window_pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut window_pid);

    if window_pid == search.pid && IsWindowVisible(hwnd) != 0 {
        if GetWindowTextLengthW(hwnd) > 0 && !search.titled_set {
            search.titled = hwnd;
            search.titled_set = true;
        }
        if !search.visible_set {
            search.visible = hwnd;
            search.visible_set = true;
        }
    }

    // Keep enumerating so we prefer the first titled window.
    1
}

/// Returns the best main-window handle for the given process id.
pub fn main_window(pid: u32) -> Option<HWND> {
    let mut search = WindowSearch {
        pid,
        ..Default::default()
    };
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut search as *mut WindowSearch as LPARAM,
        );
    }

    if search.titled_set {
        Some(search.titled)
    } else if search.visible_set {
        Some(search.visible)
    } else {
        None
    }
}

pub fn window_rect(hwnd: HWND) -> Option<Rect> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    if ok == 0 {
        return None;
    }
    Some(Rect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

#[allow(dead_code)]
pub fn foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

/// Current global cursor position, in screen coordinates.
pub fn cursor_pos() -> (i32, i32) {
    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok == 0 {
        (0, 0)
    } else {
        (point.x, point.y)
    }
}

/// Whether the primary mouse button is currently held down.
pub fn left_button_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0 }
}

/// Restores (if minimized) and focuses the given window.
///
/// Windows normally refuses to let a background process steal focus, which is
/// what stops macros from working against a game. We work around it without
/// elevation by briefly attaching our input thread to the target (and the
/// current foreground) thread before calling `SetForegroundWindow`.
pub fn set_foreground(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }

    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindowAsync(hwnd, SW_RESTORE);
        }

        let current_thread = GetCurrentThreadId();
        let target_thread = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let foreground = GetForegroundWindow();
        let foreground_thread = GetWindowThreadProcessId(foreground, std::ptr::null_mut());

        let attach_foreground = foreground_thread != 0 && foreground_thread != current_thread;
        let attach_target = target_thread != 0 && target_thread != current_thread;

        if attach_foreground {
            AttachThreadInput(current_thread, foreground_thread, 1);
        }
        if attach_target {
            AttachThreadInput(current_thread, target_thread, 1);
        }

        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        SetFocus(hwnd);

        if attach_target {
            AttachThreadInput(current_thread, target_thread, 0);
        }
        if attach_foreground {
            AttachThreadInput(current_thread, foreground_thread, 0);
        }
    }
}

/// Blocks keyboard and mouse input. Requires the process to run elevated.
#[allow(dead_code)]
pub fn block_input(block: bool) {
    unsafe {
        BlockInput(if block { 1 } else { 0 });
    }
}
