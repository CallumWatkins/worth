//! Native Snap hit testing and the Windows system menu. Vue owns the caption buttons.
//! All HWNDs and callback state are accessed exclusively on the Windows UI thread.

use std::{cell::Cell, ptr, rc::Rc};

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, WebviewWindow, plugin::TauriPlugin, webview::PageLoadEvent};
use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::{ScreenToClient, ValidateRect},
        UI::{
            HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
            Input::KeyboardAndMouse::{
                GetCapture, ReleaseCapture, SetCapture, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT,
                TrackMouseEvent,
            },
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, EnableMenuItem, GWL_STYLE, GetClientRect,
                GetCursorPos, GetPropW, GetSystemMenu, GetWindowLongW, GetWindowRect, HMENU,
                HTMAXBUTTON, HTTRANSPARENT, HWND_TOP, IDC_ARROW, IsIconic, IsWindow, IsZoomed,
                LoadCursorW, MF_BYCOMMAND, MF_ENABLED, MF_GRAYED, PostMessageW, RemovePropW,
                SC_CLOSE, SC_MAXIMIZE, SC_MINIMIZE, SC_MOVE, SC_RESTORE, SC_SIZE, SM_CYFRAME,
                SWP_NOACTIVATE, SetCursor, SetMenuDefaultItem, SetPropW, SetWindowPos,
                TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_CANCELMODE, WM_CAPTURECHANGED,
                WM_ERASEBKGND, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCDESTROY, WM_NCHITTEST,
                WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_PAINT,
                WM_SETCURSOR, WM_SYSCOMMAND, WS_CHILD, WS_EX_TRANSPARENT, WS_MAXIMIZEBOX,
                WS_MINIMIZEBOX, WS_THICKFRAME, WS_VISIBLE,
            },
        },
    },
    core::w,
};

const CHILD_PROPERTY: *const u16 = w!("Worth.WindowSnap");
const SUBCLASS_ID: usize = 0x57534e50;

#[derive(Clone, Copy, Deserialize)]
struct Bounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Bounds {
    fn fits(self, client: RECT) -> bool {
        self.x >= 0
            && self.y >= 0
            && self.width > 0
            && self.height > 0
            && i64::from(self.x) + i64::from(self.width) <= i64::from(client.right)
            && i64::from(self.y) + i64::from(self.height) <= i64::from(client.bottom)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Serialize)]
struct Interaction {
    hovered: bool,
    pressed: bool,
}

impl Interaction {
    fn release(&mut self, inside: bool) -> bool {
        let clicked = self.pressed && inside;
        self.pressed = false;
        self.hovered = inside;
        clicked
    }
}

struct Target {
    parent: HWND,
    interaction: Cell<Interaction>,
    emit: Box<dyn Fn(Interaction)>,
}

impl Target {
    fn update(&self, interaction: Interaction) {
        if self.interaction.replace(interaction) != interaction {
            (self.emit)(interaction);
        }
    }
}

pub fn init() -> TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("window-snap")
        .invoke_handler(
            tauri::generate_handler![#![plugin(window_snap)] set_bounds, show_system_menu],
        )
        .on_page_load(|webview, payload| {
            // A full WebView reload need not run Vue's unmount hooks. Never leave
            // an old hit target over the startup drag region of the next page.
            if matches!(payload.event(), PageLoadEvent::Started)
                && let Some(window) = webview.get_webview_window(webview.label())
            {
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = set_bounds(window, None).await {
                        eprintln!("Could not clear Windows Snap target on page load: {error}");
                    }
                });
            }
        })
        .build()
}

#[tauri::command]
async fn show_system_menu(window: WebviewWindow) -> Result<(), String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            let result = target
                .hwnd()
                .map_err(|error| error.to_string())
                .and_then(|handle| {
                    // SAFETY: The menu and HWND belong to this UI thread. TrackPopupMenu
                    // runs Windows' menu loop; selection is dispatched through WM_SYSCOMMAND.
                    unsafe {
                        let hwnd = handle.0 as HWND;
                        let menu = system_menu(hwnd)?;
                        let mut cursor: POINT = std::mem::zeroed();
                        if GetCursorPos(&mut cursor) == 0 {
                            return Err(std::io::Error::last_os_error().to_string());
                        }
                        let command = TrackPopupMenu(
                            menu,
                            TPM_RETURNCMD | TPM_RIGHTBUTTON,
                            cursor.x,
                            cursor.y,
                            0,
                            hwnd,
                            ptr::null(),
                        );
                        if command != 0
                            && PostMessageW(hwnd, WM_SYSCOMMAND, command as WPARAM, 0) == 0
                        {
                            return Err(std::io::Error::last_os_error().to_string());
                        }
                        Ok(())
                    }
                });
            let _ = send.send(result);
        })
        .map_err(|error| error.to_string())?;
    receive.await.map_err(|error| error.to_string())?
}

unsafe fn system_menu(hwnd: HWND) -> Result<HMENU, String> {
    // SAFETY: The caller owns a live window on this thread. Windows owns the menu.
    unsafe {
        let menu = GetSystemMenu(hwnd, 0);
        if menu.is_null() {
            return Err("The window has no system menu".into());
        }
        let maximized = IsZoomed(hwnd) != 0;
        let minimized = IsIconic(hwnd) != 0;
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        [
            (SC_RESTORE, maximized || minimized),
            (SC_MOVE, !maximized && !minimized),
            (
                SC_SIZE,
                !maximized && !minimized && style & WS_THICKFRAME != 0,
            ),
            (SC_MINIMIZE, !minimized && style & WS_MINIMIZEBOX != 0),
            (SC_MAXIMIZE, !maximized && style & WS_MAXIMIZEBOX != 0),
        ]
        .into_iter()
        .for_each(|(command, enabled)| {
            EnableMenuItem(
                menu,
                command,
                MF_BYCOMMAND | if enabled { MF_ENABLED } else { MF_GRAYED },
            );
        });
        SetMenuDefaultItem(menu, SC_CLOSE, 0);
        Ok(menu)
    }
}

#[tauri::command]
async fn set_bounds(window: WebviewWindow, bounds: Option<Bounds>) -> Result<(), String> {
    let (send, receive) = tokio::sync::oneshot::channel();
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            // SAFETY: This closure runs on the owning window's UI thread.
            let result =
                target
                    .hwnd()
                    .map_err(|error| error.to_string())
                    .and_then(|handle| unsafe {
                        position_target(
                            handle.0 as HWND,
                            bounds,
                            Box::new(move |interaction| {
                                let _ = target.emit_to(
                                    target.label(),
                                    "worth://window-snap",
                                    interaction,
                                );
                            }),
                        )
                    });
            let _ = send.send(result);
        })
        .map_err(|error| error.to_string())?;
    receive.await.map_err(|error| error.to_string())?
}

unsafe fn position_target(
    parent: HWND,
    bounds: Option<Bounds>,
    emit: Box<dyn Fn(Interaction)>,
) -> Result<(), String> {
    // SAFETY: Called only on the UI thread. Every native operation below uses this
    // live parent or its own child; failed setup is rolled back before returning.
    unsafe {
        if IsWindow(parent) == 0 {
            return Ok(());
        }
        let child = GetPropW(parent, CHILD_PROPERTY) as HWND;
        let mut client: RECT = std::mem::zeroed();
        if GetClientRect(parent, &mut client) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let Some(bounds) = bounds.filter(|bounds| bounds.fits(client)) else {
            if !child.is_null() && DestroyWindow(child) == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            return Ok(());
        };
        if !child.is_null() {
            return place_target(child, bounds);
        }

        let child = CreateWindowExW(
            WS_EX_TRANSPARENT,
            w!("STATIC"),
            ptr::null(),
            WS_CHILD | WS_VISIBLE,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            parent,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
        );
        if child.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let target = Rc::into_raw(Rc::new(Target {
            parent,
            interaction: Cell::default(),
            emit,
        }));
        if SetWindowSubclass(child, Some(target_proc), SUBCLASS_ID, target as usize) == 0 {
            let error = "Could not install the native maximize hit target".to_string();
            drop(Rc::from_raw(target));
            DestroyWindow(child);
            return Err(error);
        }
        if SetPropW(parent, CHILD_PROPERTY, child) == 0 {
            let error = std::io::Error::last_os_error().to_string();
            // WM_NCDESTROY releases the callback's reference.
            DestroyWindow(child);
            return Err(error);
        }
        // CreateWindowEx puts a new child at the bottom of the sibling Z-order.
        // Raise it immediately, even if the button's bounds never change again.
        place_target(child, bounds)
    }
}

unsafe fn place_target(child: HWND, bounds: Bounds) -> Result<(), String> {
    // SAFETY: Both callers own a live child on the window's UI thread.
    unsafe {
        if SetWindowPos(
            child,
            HWND_TOP,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            SWP_NOACTIVATE,
        ) == 0
        {
            let error = std::io::Error::last_os_error().to_string();
            DestroyWindow(child);
            return Err(error);
        }
        Ok(())
    }
}

unsafe fn cursor_inside(hwnd: HWND) -> bool {
    // SAFETY: The caller holds a live child window during its message callback.
    unsafe {
        let mut cursor: POINT = std::mem::zeroed();
        let mut rect: RECT = std::mem::zeroed();
        GetCursorPos(&mut cursor) != 0
            && GetWindowRect(hwnd, &mut rect) != 0
            && cursor.x >= rect.left
            && cursor.x < rect.right
            && cursor.y >= rect.top
            && cursor.y < rect.bottom
    }
}

unsafe extern "system" fn target_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: `data` owns one Rc reference from successful subclass installation.
    // Hold a separate reference for this callback: native calls may re-enter us,
    // including WM_NCDESTROY. No Rust mutable borrow spans a native call.
    unsafe {
        let pointer = data as *const Target;
        Rc::increment_strong_count(pointer);
        let target = Rc::from_raw(pointer);
        match message {
            WM_NCDESTROY => {
                RemoveWindowSubclass(hwnd, Some(target_proc), id);
                if GetPropW(target.parent, CHILD_PROPERTY) == hwnd {
                    RemovePropW(target.parent, CHILD_PROPERTY);
                }
                target.update(Interaction::default());
                drop(Rc::from_raw(pointer));
                return DefSubclassProc(hwnd, message, wparam, lparam);
            }
            // The child is input-only; preserve the Vue button underneath it.
            WM_PAINT => {
                ValidateRect(hwnd, ptr::null());
                return 0;
            }
            WM_ERASEBKGND => return 1,
            WM_NCHITTEST => {
                // Let Tauri's underlying native resize window receive the top
                // border. Its thickness is a per-monitor Windows metric.
                let mut point = POINT {
                    x: i32::from(lparam as u16 as i16),
                    y: i32::from((lparam >> 16) as u16 as i16),
                };
                if IsZoomed(target.parent) == 0
                    && GetWindowLongW(target.parent, GWL_STYLE) as u32 & WS_THICKFRAME != 0
                    && ScreenToClient(target.parent, &mut point) != 0
                    && point.y >= 0
                    && point.y < GetSystemMetricsForDpi(SM_CYFRAME, GetDpiForWindow(target.parent))
                {
                    return HTTRANSPARENT as LRESULT;
                }
                return HTMAXBUTTON as LRESULT;
            }
            WM_NCMOUSEMOVE | WM_MOUSEMOVE => {
                let mut interaction = target.interaction.get();
                interaction.hovered = cursor_inside(hwnd);
                target.update(interaction);
                if message == WM_NCMOUSEMOVE {
                    let mut tracking = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE | TME_NONCLIENT,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    TrackMouseEvent(&mut tracking);
                }
                return 0;
            }
            WM_NCMOUSELEAVE => {
                let mut interaction = target.interaction.get();
                interaction.hovered = false;
                target.update(interaction);
                return 0;
            }
            WM_NCLBUTTONDOWN if wparam == HTMAXBUTTON as usize => {
                target.update(Interaction {
                    hovered: true,
                    pressed: true,
                });
                SetCapture(hwnd);
                if GetCapture() != hwnd {
                    target.update(Interaction::default());
                }
                return 0;
            }
            WM_LBUTTONUP | WM_NCLBUTTONUP => {
                let mut interaction = target.interaction.get();
                let clicked = interaction.release(cursor_inside(hwnd));
                target.update(interaction);
                if GetCapture() == hwnd {
                    ReleaseCapture();
                }
                if clicked {
                    let command = if IsZoomed(target.parent) != 0 {
                        SC_RESTORE
                    } else {
                        SC_MAXIMIZE
                    };
                    PostMessageW(target.parent, WM_SYSCOMMAND, command as WPARAM, 0);
                }
                return 0;
            }
            WM_CAPTURECHANGED | WM_CANCELMODE => {
                target.update(Interaction::default());
                if message == WM_CANCELMODE && GetCapture() == hwnd {
                    ReleaseCapture();
                }
                return 0;
            }
            WM_SETCURSOR => {
                SetCursor(LoadCursorW(ptr::null_mut(), IDC_ARROW));
                return 1;
            }
            _ => {}
        }
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::{
        Graphics::Gdi::ClientToScreen,
        UI::WindowsAndMessaging::{
            GetMenuDefaultItem, GetMenuState, SetWindowLongW, WS_MAXIMIZE, WS_MINIMIZE, WS_POPUP,
            WS_SYSMENU,
        },
    };

    // Never shown; these tests must not interact with the user's running app.
    struct Parent(HWND);
    impl Parent {
        fn new() -> Self {
            unsafe {
                let hwnd = CreateWindowExW(
                    0,
                    w!("STATIC"),
                    ptr::null(),
                    WS_POPUP | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
                    -400,
                    -400,
                    1400,
                    900,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null(),
                );
                assert!(!hwnd.is_null());
                Self(hwnd)
            }
        }
    }
    impl Drop for Parent {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }

    #[test]
    fn native_target_starts_above_webview_and_removes_cleanly() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GW_CHILD, GetWindow, SendMessageW};

        unsafe {
            let parent = Parent::new();
            let webview = CreateWindowExW(
                0,
                w!("STATIC"),
                ptr::null(),
                WS_CHILD | WS_VISIBLE,
                0,
                0,
                1400,
                900,
                parent.0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            );
            assert!(!webview.is_null());
            let bounds = Bounds {
                x: 1308,
                y: 0,
                width: 46,
                height: 32,
            };
            position_target(parent.0, Some(bounds), Box::new(|_| {})).unwrap();
            let child = GetPropW(parent.0, CHILD_PROPERTY) as HWND;
            assert!(!child.is_null());
            assert_eq!(
                GetWindow(parent.0, GW_CHILD),
                child,
                "Snap target must be above the WebView immediately"
            );
            let mut origin = POINT { x: bounds.x, y: 0 };
            assert_ne!(ClientToScreen(parent.0, &mut origin), 0);
            let hit_at = |y: i32| {
                let coordinates = ((origin.y + y) as u16 as u32) << 16 | origin.x as u16 as u32;
                SendMessageW(child, WM_NCHITTEST, 0, coordinates as LPARAM)
            };
            let border = GetSystemMetricsForDpi(SM_CYFRAME, GetDpiForWindow(parent.0));
            assert_eq!(hit_at(0), HTTRANSPARENT as LRESULT);
            assert_eq!(hit_at(border - 1), HTTRANSPARENT as LRESULT);
            assert_eq!(hit_at(border), HTMAXBUTTON as LRESULT);
            let style = GetWindowLongW(parent.0, GWL_STYLE);
            SetWindowLongW(parent.0, GWL_STYLE, style | WS_MAXIMIZE as i32);
            assert_eq!(hit_at(0), HTMAXBUTTON as LRESULT);
            SetWindowLongW(parent.0, GWL_STYLE, style);
            position_target(parent.0, None, Box::new(|_| {})).unwrap();
            assert!(GetPropW(parent.0, CHILD_PROPERTY).is_null());
            assert_eq!(IsWindow(child), 0);
            assert_eq!(GetWindow(parent.0, GW_CHILD), webview);
            position_target(parent.0, Some(bounds), Box::new(|_| {})).unwrap();
            assert!(!GetPropW(parent.0, CHILD_PROPERTY).is_null());
        }
    }

    #[test]
    fn system_menu_tracks_window_state() {
        unsafe {
            let parent = Parent::new();
            let style = GetWindowLongW(parent.0, GWL_STYLE);
            let menu = system_menu(parent.0).unwrap();
            let enabled = |command| GetMenuState(menu, command, MF_BYCOMMAND) & MF_GRAYED == 0;
            assert!(enabled(SC_SIZE) && enabled(SC_MOVE) && enabled(SC_MAXIMIZE));
            assert!(!enabled(SC_RESTORE));
            assert_eq!(GetMenuDefaultItem(menu, 0, 0), SC_CLOSE);

            SetWindowLongW(parent.0, GWL_STYLE, style | WS_MAXIMIZE as i32);
            system_menu(parent.0).unwrap();
            assert!(!enabled(SC_SIZE) && !enabled(SC_MOVE) && !enabled(SC_MAXIMIZE));
            assert!(enabled(SC_RESTORE) && enabled(SC_MINIMIZE));

            SetWindowLongW(parent.0, GWL_STYLE, style | WS_MINIMIZE as i32);
            system_menu(parent.0).unwrap();
            assert!(enabled(SC_RESTORE));
            assert!(!enabled(SC_MINIMIZE));
        }
    }

    #[test]
    fn only_a_press_followed_by_release_inside_clicks() {
        let mut state = Interaction::default();
        assert!(!state.release(true));
        state.pressed = true;
        assert!(!state.release(false));
        assert!(!state.release(true));
        state.pressed = true;
        assert!(state.release(true));
        assert!(!state.release(true));
    }

    #[test]
    fn bounds_must_be_visible_and_inside_the_client_area() {
        let client = RECT {
            left: 0,
            top: 0,
            right: 1400,
            bottom: 900,
        };
        let valid = Bounds {
            x: 1308,
            y: 0,
            width: 46,
            height: 32,
        };
        assert!(valid.fits(client));
        assert!(!Bounds { width: 0, ..valid }.fits(client));
        assert!(!Bounds { x: -1, ..valid }.fits(client));
        assert!(
            !Bounds {
                x: i32::MAX,
                ..valid
            }
            .fits(client)
        );
        assert!(
            !Bounds {
                height: 901,
                ..valid
            }
            .fits(client)
        );
    }
}
