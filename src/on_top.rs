//! 窗口保持在最前 (#127): the "…" menu (and on Windows a button in the title
//! bar) floats this window above other apps' windows, like a sticky note, until
//! it is turned off or the window closes. Not remembered: a window left above
//! everything is easy to forget, so it is turned on each time.
use crate::app::Workspace;
use crate::ui::{TipExt, TitleBarExt};
use gpui::*;

/// A layer in front of others (Nucleo LayerFront), the menu row's and the
/// title bar's: the pin is 置顶's, which pins a document, not the window.
pub const ICON: &str = "layer-front";

/// Whether this platform keeps a window on top: macOS and Windows. Linux hides
/// it: GPUI sets a window's level only when making it, and Wayland leaves
/// stacking to the compositor.
pub const SUPPORTED: bool = cfg!(any(target_os = "macos", target_os = "windows"));

/// Float `window` above other apps' windows, or put it back among them.
pub fn set(window: &Window, on: bool) {
    let handle = crate::printing::parent_of(window);
    if handle == 0 { return }
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};
        let win: *mut Object = msg_send![handle as *mut Object, window];
        if win.is_null() { return }
        // NSFloatingWindowLevel, or back to NSNormalWindowLevel
        let _: () = msg_send![win, setLevel: if on { 3isize } else { 0isize }];
    }
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos};
        if let Err(e) = SetWindowPos(HWND(handle as *mut std::ffi::c_void), Some(if on { HWND_TOPMOST } else { HWND_NOTOPMOST }), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) {
            log::warn!("on top: {e}");
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = on;
}

impl Workspace {
    /// The one switch the "…" menu and the title bar's button both turn.
    pub fn set_on_top(&mut self, on: bool, window: &Window, cx: &mut Context<Self>) {
        self.on_top = on;
        set(window, on);
        cx.notify();
    }
}

/// Windows' title bar button, left of minimize: lit while on. Linux
/// draws its own title bar too but can't keep a window on top; macOS has the
/// menu only.
pub fn title_button(on: bool, cx: &mut Context<Workspace>) -> Option<AnyElement> {
    if !cfg!(target_os = "windows") { return None }
    Some(crate::design::Button::ghost("window-on-top").w(px(crate::app::HEADER_ACTIONS_H)).h(px(crate::app::HEADER_ACTIONS_H)).mx(px(7.)).flex_none()
        .icon(ICON).selected(on).no_drag()
        .tip(crate::settings::tr("窗口保持在最前", "Keep Window on Top"), None)
        .on_click(cx.listener(move |w, _, window, cx| w.set_on_top(!on, window, cx)))
        .into_any_element())
}
