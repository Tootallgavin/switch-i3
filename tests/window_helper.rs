extern crate x11_dl;

use std::ffi::CString;
use std::mem;
use std::os::raw::*;
use std::ptr;

use self::x11_dl::xlib;

pub struct WindowHelper {
    display: *mut x11_dl::xlib::_XDisplay,
    xlib: xlib::Xlib,
}

impl WindowHelper {
    pub fn close_window(&self) {
        unsafe {
            // Shut down.
            (self.xlib.XCloseDisplay)(self.display);
        }
    }
    pub fn create_window_with_name(name: &str) -> WindowHelper {
        unsafe {
            // Load Xlib library.
            let xlib = xlib::Xlib::open().unwrap();

            //   Open display connection.
            let display = (xlib.XOpenDisplay)(ptr::null());

            let mut wh = WindowHelper {
                display: display,
                xlib: xlib,
            };

            wh.open_window_with_name(name);

            return wh;
        }
    }
    fn open_window_with_name(&mut self, name: &str) {
        unsafe {
            //
            if self.display.is_null() {
                panic!("XOpenDisplay failed");
            }
            //
            //       // Create window.
            let screen = (self.xlib.XDefaultScreen)(self.display);
            let root = (self.xlib.XRootWindow)(self.display, screen);
            //
            let mut attributes: self::xlib::XSetWindowAttributes = mem::uninitialized();
            attributes.background_pixel = (self.xlib.XWhitePixel)(self.display, screen);
            //
            let window = (self.xlib.XCreateWindow)(
                self.display,
                root,
                0,
                0,
                400,
                300,
                0,
                0,
                xlib::InputOutput as c_uint,
                ptr::null_mut(),
                xlib::CWBackPixel,
                &mut attributes,
            );
            // Set window title.
            let title_str = CString::new(name).unwrap();
            (self.xlib.XStoreName)(self.display, window, title_str.as_ptr() as *mut c_char);

            // Hook close requests.
            let wm_protocols_str = CString::new("WM_PROTOCOLS").unwrap();
            let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();

            let wm_protocols =
                (self.xlib.XInternAtom)(self.display, wm_protocols_str.as_ptr(), xlib::False);
            let wm_delete_window =
                (self.xlib.XInternAtom)(self.display, wm_delete_window_str.as_ptr(), xlib::False);

            let mut protocols = [wm_delete_window];
            (self.xlib.XSetWMProtocols)(
                self.display,
                window,
                protocols.as_mut_ptr(),
                protocols.len() as c_int,
            );
            //
            // // Show window.
            (self.xlib.XMapWindow)(self.display, window);

            // Main loop.
            let mut event: xlib::XEvent = mem::uninitialized();
            (self.xlib.XCheckIfEvent)(self.display, &mut event, None, ptr::null_mut());
        }
    }
}
