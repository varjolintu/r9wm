/*
* The MIT License (MIT)
*
* Based on rust-tinywm: https://github.com/acmiyaguchi/rust-tinywm
* Inspired by uwurawrxdwm: https://github.com/vardy/uwurawrxdwm
*
* Copyright (c) 2014 Anthony Miyaguchi <acmiyaguchi@gmail.com>
* Copyright (c) 2018 Sami Vänttinen <sami.vanttinen@protonmail.com>
*/

extern crate libc;
extern crate x11;

use std::{error::Error, mem::zeroed, process::Command};
use libc::{c_int, c_uint};
use x11::{xlib::*, keysym::*};
use std::ffi::OsStr;

fn max(a : c_int, b : c_int) -> c_uint { if a > b { a as c_uint } else { b as c_uint } }

fn spawn_process(process: &OsStr) {
	match Command::new(process).spawn() {
		Err(e) => eprintln!("couldn't spawn: {}", e.description()),
		_ => {}
	};
}

fn main() {
    let mut arg0 = 0x0 as i8;
    let display : *mut Display = unsafe { XOpenDisplay(&mut arg0) };
    let mut attr: XWindowAttributes = unsafe { zeroed() };
    let mut start: XButtonEvent = unsafe { zeroed() };
    let mut window: Window = unsafe { zeroed() };
    let mut revert_to: i32 = 0;
    //let mut cursor: Cursor = unsafe { zeroed() };

    if display.is_null() {
        std::process::exit(1);
    }

    unsafe {
    	let shortcuts: Vec<c_uint> = vec![XK_d, XK_q, XK_Return, XK_space, XK_BackSpace];
	    for key in shortcuts {
			XGrabKey(display, XKeysymToKeycode(display, key.into()) as c_int, Mod1Mask, XDefaultRootWindow(display), true as c_int, GrabModeAsync, GrabModeAsync);
		}	

        XGrabButton(display, 1, Mod1Mask, XDefaultRootWindow(display), true as c_int,
        	        (ButtonPressMask|ButtonReleaseMask|PointerMotionMask) as c_uint, GrabModeAsync, GrabModeAsync, 0, 0);
        XGrabButton(display, 3, Mod1Mask, XDefaultRootWindow(display), true as c_int,
        	        (ButtonPressMask|ButtonReleaseMask|PointerMotionMask) as c_uint, GrabModeAsync, GrabModeAsync, 0, 0);
    };

    start.subwindow = 0;
    let mut event: XEvent = unsafe { zeroed() };

    loop {
        unsafe {
            XNextEvent(display, &mut event);
            XGetInputFocus(display, &mut window, &mut revert_to);

            match event.get_type() {
                x11::xlib::KeyPress => {
                    let xkey: XKeyEvent = From::from(event);
                    if xkey.subwindow != 0 {
                        XRaiseWindow(display, xkey.subwindow);

                        // Close window with mod+q
	            		if event.key.keycode == XKeysymToKeycode(display, x11::keysym::XK_q.into()).into() {
	            			XDestroyWindow(display, window);
	            		}
                    }

                    // Open a terminal with mod+enter
                    if event.key.keycode == XKeysymToKeycode(display, XK_Return.into()).into() {
            			spawn_process(OsStr::new("urxvt"));
            		}

                    // Open dmenu with mod+d
            		if event.key.keycode == XKeysymToKeycode(display, XK_d.into()).into() {
            			spawn_process(OsStr::new("dmenu_run"));
            		}

            		// Open rofi with mod+space
            		if event.key.keycode == XKeysymToKeycode(display, XK_space.into()).into() {
            			match Command::new("rofi").args(&["-show", "run"]).spawn()  {
            				Err(e) => eprintln!("couldn't spawn: {}", e.description()),
        					_ => {}
            			};
            		}

            		// Close r9wm with mod+backspace
            		if event.key.keycode == XKeysymToKeycode(display, XK_BackSpace.into()).into() {
            			XCloseDisplay(display);
            		}
                },
                x11::xlib::ButtonPress => {
                    let xbutton: XButtonEvent = From::from(event);
                    if xbutton.subwindow != 0 {
                        XGetWindowAttributes(display, xbutton.subwindow, &mut attr);
                        start = xbutton;
                    }
                },
                x11::xlib::MotionNotify => {
                    if start.subwindow != 0 {
                    	//cursor = XCreateFontCursor(display, 58);
                    	//XDefineCursor(display, start.subwindow, cursor);
                        let xbutton: XButtonEvent = From::from(event);
                        let xdiff: c_int = xbutton.x_root - start.x_root;
                        let ydiff: c_int = xbutton.y_root - start.y_root;
                        XMoveResizeWindow(display, start.subwindow,
                                          attr.x + (if start.button == 1 { xdiff } else { 0 }),
                                          attr.y + (if start.button == 1 { ydiff } else { 0 }),
                                          max(1, attr.width + (if start.button == 3 { xdiff } else { 0 })),
                                          max(1, attr.height + (if start.button == 3 { ydiff } else { 0 })));
                    }
                },
                x11::xlib::ButtonRelease => {
                    start.subwindow = 0;
                },
                _ => {}
            };
        }
    }
}
