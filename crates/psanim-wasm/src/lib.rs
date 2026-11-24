//! psanim for the browser.
//!
//! A plain C ABI over [`psanim::Movie`], with no JavaScript glue generator:
//! the page instantiates the `.wasm` file directly and calls these exports
//! (see `web/psanim.js`). One movie is loaded at a time.
//!
//! Strings cross the boundary as UTF-8 in the module's memory: the page
//! writes into a buffer from [`pa_alloc`], and reads results through a
//! `*_ptr` / `*_len` pair. Pixels come back as straight (not
//! premultiplied) RGBA, which is what a canvas `ImageData` holds.
//!
//! The engine here is the same code as the native one. What the browser
//! build does not have is wall-clock limits (there is no clock in
//! `wasm32-unknown-unknown`; step limits work), threads, and file access:
//! a program can read PSCat's built-in libraries and nothing else.

use std::cell::RefCell;

use psanim::{Budget, Limits, Movie, Options, Scale};

#[derive(Default)]
struct State {
    movie: Option<Movie>,
    pixels: Vec<u8>,
    size: (u32, u32),
    /// The last error, or the last successful result, as JSON.
    error: String,
    meta: String,
    text: String,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// Read a string the page wrote into memory from [`pa_alloc`].
///
/// # Safety
/// `ptr` must point to `len` readable bytes.
unsafe fn text<'a>(ptr: *const u8, len: usize) -> &'a str {
    if ptr.is_null() || len == 0 {
        return "";
    }
    // SAFETY: the caller's contract.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes).unwrap_or("")
}

/// Allocate `len` bytes for the page to write a string into.
#[unsafe(no_mangle)]
pub extern "C" fn pa_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len.max(1));
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free a buffer from [`pa_alloc`].
///
/// # Safety
/// `ptr` must come from `pa_alloc(len)` and not have been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pa_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        // SAFETY: the caller's contract: same pointer and capacity.
        drop(unsafe { Vec::from_raw_parts(ptr, 0, len.max(1)) });
    }
}

/// Load a program. `scale` is pixels per canvas unit (0 for 1.0);
/// `max_steps` caps the interpreter steps of setup and of each frame (0 for
/// no cap). Returns 0 on success; on failure [`pa_error_ptr`] has the
/// error as JSON.
///
/// # Safety
/// `src` must point to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pa_load(src: *const u8, len: usize, scale: f64, max_steps: f64) -> i32 {
    // SAFETY: the caller's contract.
    let source = unsafe { text(src, len) };
    let limits = if max_steps >= 1.0 {
        Limits::none().with_max_steps(max_steps as u64)
    } else {
        Limits::none()
    };
    let options = Options {
        scale: Scale::Factor(if scale > 0.0 { scale } else { 1.0 }),
        budget: Budget {
            setup: limits,
            frame: limits,
        },
        // No disk in a browser: only the built-in libraries.
        read_roots: Some(Vec::new()),
        ..Options::default()
    };
    with(|s| {
        s.movie = None;
        match Movie::load(source, &options) {
            Ok(movie) => {
                s.meta = movie.meta().to_json().to_string();
                s.error.clear();
                s.movie = Some(movie);
                0
            }
            Err(e) => {
                s.error = e.to_json().to_string();
                s.text = e.to_string();
                1
            }
        }
    })
}

/// Render the moment `time` seconds in. Returns 0 on success, after which
/// [`pa_pixels_ptr`] holds `pa_width() * pa_height() * 4` bytes of straight
/// RGBA.
#[unsafe(no_mangle)]
pub extern "C" fn pa_render(time: f64) -> i32 {
    with(|s| {
        let Some(movie) = s.movie.as_mut() else {
            s.error = r#"{"phase":"load","kind":"nomovie","message":"no movie is loaded"}"#.into();
            s.text = "no movie is loaded".into();
            return 1;
        };
        match movie.render_at(time) {
            Ok(frame) => {
                s.size = (frame.width, frame.height);
                s.pixels = frame.to_rgba8();
                s.error.clear();
                0
            }
            Err(e) => {
                s.error = e.to_json().to_string();
                s.text = e.to_string();
                1
            }
        }
    })
}

/// Drop the loaded movie and free its memory.
#[unsafe(no_mangle)]
pub extern "C" fn pa_unload() {
    with(|s| {
        s.movie = None;
        s.pixels = Vec::new();
    });
}

/// Set the signal `name` for the frames rendered from now on: the page
/// calls this with the pointer position before each frame.
///
/// # Safety
/// `name` must point to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pa_set_signal(name: *const u8, len: usize, value: f64) {
    // SAFETY: the caller's contract.
    let name = unsafe { text(name, len) };
    with(|s| {
        if let Some(m) = s.movie.as_mut() {
            m.set_signal(name, value);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_pixels_ptr() -> *const u8 {
    with(|s| s.pixels.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_width() -> u32 {
    with(|s| s.size.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_height() -> u32 {
    with(|s| s.size.1)
}

/// The loaded movie's metadata as JSON (`psanim info --json`'s `movie`).
#[unsafe(no_mangle)]
pub extern "C" fn pa_meta_ptr() -> *const u8 {
    with(|s| s.meta.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_meta_len() -> usize {
    with(|s| s.meta.len())
}

/// The last error as JSON; empty after a success.
#[unsafe(no_mangle)]
pub extern "C" fn pa_error_ptr() -> *const u8 {
    with(|s| s.error.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_error_len() -> usize {
    with(|s| s.error.len())
}

/// The last error as the text the command-line tool prints.
#[unsafe(no_mangle)]
pub extern "C" fn pa_error_text_ptr() -> *const u8 {
    with(|s| s.text.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn pa_error_text_len() -> usize {
    with(|s| s.text.len())
}

/// Interpreter steps the last frame took (when a step cap is set).
#[unsafe(no_mangle)]
pub extern "C" fn pa_last_steps() -> f64 {
    with(|s| {
        s.movie
            .as_ref()
            .map_or(0.0, |m| m.last_frame_steps() as f64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(ptr: *const u8, len: usize) -> String {
        // SAFETY: the exports return pointers into live state.
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned()
    }

    /// The same calls the page makes, natively.
    #[test]
    fn the_abi_loads_renders_and_reports() {
        let src = "<< /Size [32 16] /Duration 2 /FPS 10 >> animation
                   /scene { /mousex signal 0 0 setrgbcolor now 16 mul 0 4 16 rectfill } def";
        let buf = pa_alloc(src.len());
        // SAFETY: `buf` has `src.len()` bytes.
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), buf, src.len()) };
        // SAFETY: `buf` holds the source.
        assert_eq!(unsafe { pa_load(buf, src.len(), 2.0, 1e6) }, 0);
        // SAFETY: allocated above with this length.
        unsafe { pa_free(buf, src.len()) };
        let meta = string(pa_meta_ptr(), pa_meta_len());
        assert!(meta.contains("\"pixel_width\":64"), "{meta}");
        assert_eq!(pa_error_len(), 0);

        let name = "mousex";
        // SAFETY: `name` is a live string.
        unsafe { pa_set_signal(name.as_ptr(), name.len(), 1.0) };
        assert_eq!(pa_render(1.0), 0);
        assert_eq!((pa_width(), pa_height()), (64, 32));
        // SAFETY: the pixel buffer is width*height*4 bytes.
        let px = unsafe { std::slice::from_raw_parts(pa_pixels_ptr(), 64 * 32 * 4) };
        // At t=1 the bar starts at x=16 units = 32 px.
        assert_eq!(
            &px[(16 * 64 + 34) * 4..(16 * 64 + 34) * 4 + 4],
            &[255, 0, 0, 255]
        );
        assert_eq!(
            &px[(16 * 64 + 10) * 4..(16 * 64 + 10) * 4 + 4],
            &[0, 0, 0, 255]
        );
        assert!(pa_last_steps() > 0.0);

        let bad = "<< /Size [32 16] >> animation /scene { nosuchword } def";
        // SAFETY: `bad` is a live string.
        assert_eq!(unsafe { pa_load(bad.as_ptr(), bad.len(), 0.0, 0.0) }, 0);
        assert_eq!(pa_render(0.0), 1);
        let err = string(pa_error_ptr(), pa_error_len());
        assert!(
            err.contains("\"kind\":\"undefined\"") && err.contains("nosuchword"),
            "{err}"
        );
        assert!(string(pa_error_text_ptr(), pa_error_text_len()).contains("frame 0"));
        // A native thread must not leave an interpreter for thread-local
        // teardown to drop (the interpreter's own thread-locals may go
        // first); the browser has no such teardown.
        pa_unload();
    }
}
