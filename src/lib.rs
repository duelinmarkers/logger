#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/logger")]
#![crate_name = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate iron;
extern crate http;
extern crate time;
extern crate term;

use iron::{Middleware, Request, Response, Status, Continue};
use time::precise_time_ns;
use term::{Terminal, WriterWrapper, stdout};

use std::io::IoResult;

use format::{Format, FormatText, Str, Method, URI, Status, ResponseTime,
             ConstantColor, FunctionColor, ConstantAttrs, FunctionAttrs};

pub mod format;

/// `Middleware` for logging request and response info to the terminal.
#[deriving(Clone)]
pub struct Logger {
    entry_time: u64,
    format: Option<Format>
}

impl Logger {
    /// Create a new `Logger` with the specified `format`. If a `None` is passed in, uses the default format:
    ///
    /// ```
    /// {method} {uri} -> {status} ({response_time} ms)
    /// ```
    pub fn new(format: Option<Format>) -> Logger {
        Logger { entry_time: 0u64, format: format }
    }
}

impl Middleware for Logger {
    fn enter(&mut self, _: &mut Request, _: &mut Response) -> Status {
        self.entry_time = precise_time_ns();
        Continue
    }
    fn exit(&mut self, req: &mut Request, res: &mut Response) -> Status {
        let response_time_ms = (precise_time_ns() - self.entry_time) as f64 / 1000000.0;
        let Format(format) = self.format.clone().unwrap_or(Format::default());

        let render = |text: &FormatText| {
            match *text {
                Str(ref string) => string.clone(),
                Method => format!("{}", req.method),
                URI => format!("{}", req.url),
                Status => format!("{}", res.status),
                ResponseTime => format!("{} ms", response_time_ms)
            }
        };
        let log = |mut t: Box<Terminal<WriterWrapper> + Send>| -> IoResult<()> {
            for unit in format.iter() {
                match unit.color {
                    ConstantColor(Some(color)) => { try!(t.fg(color)); }
                    ConstantColor(None) => (),
                    FunctionColor(f) => match f(req, res) {
                        Some(color) => { try!(t.fg(color)); }
                        None => ()
                    }
                }
                match unit.attrs {
                    ConstantAttrs(ref attrs) => {
                        for &attr in attrs.iter() {
                            try!(t.attr(attr));
                        }
                    }
                    FunctionAttrs(f) => {
                        for &attr in f(req, res).iter() {
                            try!(t.attr(attr));
                        }
                    }
                }
                try!(write!(t, "{}", render(&unit.text)));
                try!(t.reset());
            }
            try!(writeln!(t, ""));
            Ok(())
        };

        match stdout() {
            Some(terminal) => {
                match log(terminal) {
                    Err(e) => { println!("Error logging to terminal: {}", e); },
                    Ok(_) => ()
                }
            }
            None => { println!("Logger could not open terminal"); }
        }
        Continue
    }
}
