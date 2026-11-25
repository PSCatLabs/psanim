//! psanim: programmable animation in PostScript.
//!
//! An animation is a PostScript program that declares a movie and defines a
//! `scene` procedure. psanim runs the program on the PSCat interpreter and
//! calls `scene` once per frame with the clock set; the program reads the
//! time with `now` and draws with everything PostScript and PSCat's
//! libraries have.
//!
//! ```
//! use psanim::{Movie, Options};
//!
//! let source = r#"
//!     << /Size [320 180] /Duration 2 /FPS 24 >> animation
//!     /scene {
//!         160 90 translate
//!         now 90 mul rotate
//!         0.9 0.3 0.1 setrgbcolor
//!         -40 -40 80 80 rectfill
//!     } def
//! "#;
//! let mut movie = Movie::load(source, &Options::default())?;
//! assert_eq!(movie.meta().frames, 48);
//! // Any frame, in any order: a frame depends on its time alone.
//! let late = movie.render_frame(40)?.premultiplied.to_vec();
//! let early = movie.render_frame(3)?.premultiplied.to_vec();
//! assert_ne!(late, early);
//! assert_eq!(movie.render_frame(40)?.premultiplied, &late[..]);
//! # Ok::<(), psanim::Error>(())
//! ```
//!
//! # What is here
//!
//! - [`Movie`] loads a program and renders frames; [`Options`] is what a
//!   host can override; [`Meta`] is what the movie turned out to be.
//! - [`api`] is the animation vocabulary as data, the reference's source.
//! - [`render`] renders ranges of frames on several threads.
//! - [`sheet`] lays frames out as a contact sheet.
//! - [`raster`], [`ease`], [`pathmath`] and [`signals`] are the pure
//!   building blocks, usable on their own.
//!
//! The crate has no platform dependencies beyond the PSCat library: it
//! does not spawn processes, encode video, or (except for [`render`]'s
//! threads) need an operating system. Video encoding lives in the `psanim`
//! command-line tool; the browser build is `psanim-wasm`.

#![allow(clippy::result_large_err)] // errors are the cold path; a rich one beats a boxed one

pub mod api;
pub mod code;
mod ctx;
pub mod ease;
mod engine;
mod error;
pub mod motion;
mod ops;
pub mod pathmath;
pub mod raster;
#[cfg(not(target_arch = "wasm32"))]
pub mod render;
pub mod sheet;
pub mod signals;
mod vocab;

pub use engine::{
    Budget, Frame, Meta, Mode, MotionBlur, Movie, Options, PRELUDE, Param, Printed, Scale,
};
pub use error::{Error, Phase, TraceLine};
pub use ops::TRANSITIONS;
pub use signals::Signals;
pub use vocab::LIBRARIES;

/// The PSCat types that appear in this crate's API, re-exported so a host
/// needs only one dependency.
pub use pscat::{Limits, Page};
