//! 使用 EXE 内嵌 ICO 同时设置标题栏与任务栏图标。

use windows_sys::Win32::{
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        GetSystemMetrics, LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IMAGE_ICON,
        LR_SHARED, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, WM_SETICON,
    },
};

pub fn apply(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let hwnd = window.hwnd()?.0;
    // Tauri 的 set_icon 仅设置 ICON_SMALL；任务栏需要独立设置 ICON_BIG。
    // 32512 是 tauri-build 的 ICO 资源编号；LR_SHARED 图标由 Windows 管理生命周期。
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        if module.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        for (kind, width, height) in [
            (ICON_SMALL, SM_CXSMICON, SM_CYSMICON),
            (ICON_BIG, SM_CXICON, SM_CYICON),
        ] {
            let icon = LoadImageW(module, 32512usize as *const u16, IMAGE_ICON,
                GetSystemMetrics(width), GetSystemMetrics(height), LR_SHARED);
            if icon.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            SendMessageW(hwnd, WM_SETICON, kind as usize, icon as isize);
        }
    }
    Ok(())
}
