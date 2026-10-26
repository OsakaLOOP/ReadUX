//! 默认使用 EXE 内嵌图标，阅读时按封面替换窗口的大、小图标。

use std::sync::Mutex;
use windows_sys::Win32::{
    Foundation::HWND,
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateIcon, DestroyIcon, GetSystemMetrics, LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL,
        IMAGE_ICON, LR_SHARED, SM_CXICON, SM_CXSMICON, SM_CYICON, SM_CYSMICON, WM_SETICON,
    },
};

pub fn apply(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    apply_embedded(window.hwnd()?.0)
}

fn apply_embedded(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
    // 任务栏可能选用大图标或小图标，两者保持相同的阅读状态。
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
            let icon = LoadImageW(
                module,
                32512usize as *const u16,
                IMAGE_ICON,
                GetSystemMetrics(width),
                GetSystemMetrics(height),
                LR_SHARED,
            );
            if icon.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            SendMessageW(hwnd, WM_SETICON, kind as usize, icon as isize);
        }
    }
    Ok(())
}

struct OwnedIcon(usize);

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.0 as _);
        }
    }
}

#[derive(Default)]
pub struct TaskbarIconState(Mutex<Option<OwnedIcon>>);

fn create_icon(mut rgba: Vec<u8>, size: u32) -> Result<OwnedIcon, String> {
    if !(16..=256).contains(&size) || rgba.len() != size as usize * size as usize * 4 {
        return Err("任务栏图标尺寸或像素数据无效".into());
    }
    // 与 Tao 的 Windows 图标路径一致，按 BGRA 和透明度掩码创建 HICON。
    let mut mask = Vec::with_capacity(rgba.len() / 4);
    for pixel in rgba.chunks_exact_mut(4) {
        mask.push(pixel[3].wrapping_sub(u8::MAX));
        pixel.swap(0, 2);
    }
    let handle = unsafe {
        CreateIcon(
            std::ptr::null_mut(),
            size as i32,
            size as i32,
            1,
            32,
            mask.as_ptr(),
            rgba.as_ptr(),
        )
    };
    if handle.is_null() {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(OwnedIcon(handle as usize))
    }
}

fn replace(
    hwnd: HWND,
    state: &TaskbarIconState,
    rgba: Option<Vec<u8>>,
    size: u32,
) -> Result<(), String> {
    let mut current = state.0.lock().map_err(|error| error.to_string())?;
    if let Some(rgba) = rgba {
        let icon = create_icon(rgba, size)?;
        for kind in [ICON_BIG, ICON_SMALL] {
            unsafe {
                SendMessageW(hwnd, WM_SETICON, kind as usize, icon.0 as isize);
            }
        }
        // Windows 已开始使用新图标，随后释放本应用持有的上一张图标。
        *current = Some(icon);
    } else {
        apply_embedded(hwnd).map_err(|error| error.to_string())?;
        *current = None;
    }
    Ok(())
}

pub fn update(
    window: &tauri::WebviewWindow,
    state: &TaskbarIconState,
    rgba: Option<Vec<u8>>,
    size: u32,
) -> Result<(), String> {
    replace(
        window.hwnd().map_err(|error| error.to_string())?.0,
        state,
        rgba,
        size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WM_GETICON};

    #[test]
    fn taskbar_icon_replaces_both_sizes() {
        unsafe {
            let hwnd = CreateWindowExW(
                0,
                windows_sys::w!("STATIC"),
                windows_sys::w!("ReadUX 图标测试"),
                0,
                0,
                0,
                100,
                100,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!hwnd.is_null());
            let default = LoadImageW(
                std::ptr::null_mut(),
                32512usize as *const u16,
                IMAGE_ICON,
                32,
                32,
                LR_SHARED,
            );
            assert!(!default.is_null());
            for kind in [ICON_BIG, ICON_SMALL] {
                SendMessageW(hwnd, WM_SETICON, kind as usize, default as isize);
            }
            let original = SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0);
            let state = TaskbarIconState::default();
            replace(hwnd, &state, Some(vec![255; 32 * 32 * 4]), 32).unwrap();
            let first = SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0);
            assert_ne!(first, 0);
            assert_ne!(first, original);
            assert_eq!(
                SendMessageW(hwnd, WM_GETICON, ICON_SMALL as usize, 0),
                first
            );
            replace(hwnd, &state, Some(vec![128; 32 * 32 * 4]), 32).unwrap();
            let second = SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0);
            assert_ne!(second, first);
            assert_eq!(
                SendMessageW(hwnd, WM_GETICON, ICON_SMALL as usize, 0),
                second
            );
            DestroyWindow(hwnd);
        }
    }

    #[test]
    fn invalid_icon_preserves_current_icon() {
        assert!(create_icon(vec![0; 4], 256).is_err());
        assert!(create_icon(Vec::new(), u32::MAX).is_err());
    }
}
