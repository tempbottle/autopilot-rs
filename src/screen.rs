//! This module contains functions for working with the screen.
extern crate image;
use bitmap;
use geometry::{Point, Rect, Size};
use self::image::{GenericImage, ImageResult, Rgba};

#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;
#[cfg(target_os = "linux")]
use internal;
#[cfg(target_os = "linux")]
use x11;

/// Returns the size of the main screen in points.
pub fn size() -> Size {
    system_size()
}

/// Returns the scale of the main screen, i.e. how many pixels are in a point.
pub fn scale() -> f64 {
    system_scale()
}

/// Returns whether the given point is inside the main screen boundaries.
pub fn is_point_visible(point: Point) -> bool {
    Rect::new(Point::ZERO, size()).is_point_visible(point)
}

/// Returns whether the given rect is inside the main screen boundaries.
pub fn is_rect_visible(rect: Rect) -> bool {
    Rect::new(Point::ZERO, size()).is_rect_visible(rect)
}

/// A convenience method that returns the RGB color at the given point on the
/// main display.
pub fn get_color(point: Point) -> ImageResult<Rgba<u8>> {
    let bmp = try!(bitmap::capture_screen_portion(Rect::new(
        point,
        Size::new(1.0, 1.0)
    )));
    Ok(bmp.image.get_pixel(0, 0))
}

#[cfg(target_os = "macos")]
fn system_size() -> Size {
    Size::from(CGDisplay::main().bounds().size)
}

#[cfg(target_os = "macos")]
fn system_scale() -> f64 {
    let mode = CGDisplay::main().display_mode().unwrap();
    mode.pixel_height() as f64 / mode.height() as f64
}

#[cfg(windows)]
fn system_size() -> Size {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    let scale_factor = scale();
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f64;
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as f64;
    Size::new(width, height).scaled(1.0 / scale_factor)
}

#[cfg(windows)]
fn system_scale() -> f64 {
    use winapi::um::winuser::GetDesktopWindow;
    use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
    use winapi::shared::minwindef::FARPROC;
    use std::ffi::CString;
    use std;
    let user32_module = unsafe { LoadLibraryA(CString::new("user32.dll").unwrap().as_ptr()) };
    let set_process_dpi_aware_ptr: FARPROC = unsafe {
        GetProcAddress(
            user32_module,
            CString::new("SetProcessDPIAware").unwrap().as_ptr(),
        )
    };
    let get_dpi_for_window_ptr: FARPROC = unsafe {
        GetProcAddress(
            user32_module,
            CString::new("GetDpiForWindow").unwrap().as_ptr(),
        )
    };

    // Guard against old Windows versions.
    if set_process_dpi_aware_ptr != std::ptr::null_mut()
        && get_dpi_for_window_ptr != std::ptr::null_mut()
    {
        let set_process_dpi_aware: SetProcessDPIAwareSignature =
            unsafe { std::mem::transmute(set_process_dpi_aware_ptr) };
        let get_dpi_for_window: GetDPIForWindowSignature =
            unsafe { std::mem::transmute(get_dpi_for_window_ptr) };
        unsafe { set_process_dpi_aware() };
        let window = unsafe { GetDesktopWindow() };
        let dpi = unsafe { get_dpi_for_window(window) };
        dpi as f64 / 96.0
    } else {
        1.0
    }
}

#[cfg(target_os = "linux")]
fn system_size() -> Size {
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
        let scale_factor = scale();
        let screen = x11::xlib::XDefaultScreen(*display);
        let width = x11::xlib::XDisplayWidth(*display, screen) as f64;
        let height = x11::xlib::XDisplayHeight(*display, screen) as f64;
        Size::new(width, height).scaled(1.0 / scale_factor)
    })
}

#[cfg(target_os = "linux")]
fn system_scale() -> f64 {
    internal::X_SCALE_FACTOR.with(|scale| *scale)
}

#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use libc;

#[cfg(windows)]
type SetProcessDPIAwareSignature = unsafe extern "C" fn();
#[cfg(windows)]
type GetDPIForWindowSignature = unsafe extern "C" fn(HWND) -> libc::c_uint;
