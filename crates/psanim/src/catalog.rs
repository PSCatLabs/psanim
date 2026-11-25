//! The catalog: ready-made animated elements, compiled in.
//!
//! Each entry is a movie file (`crates/psanim/catalog/NAME.ps`) with its
//! own declaration, parameters and scene. It runs on its own (`psanim
//! check`, `psanim sheet`), and a movie places it as a component:
//! `/busy /spinner component`, then `/busy << /Box [..] >> place`.

use pscat::Interp;
use pscat::PsError;
use pscat::host::Value;

use crate::ops::{Shared, fail, reg};

/// Every catalog entry: its name and its source.
pub const CATALOG: &[(&str, &str)] = &[
    ("clock-analog", include_str!("../catalog/clock-analog.ps")),
    ("clock-digital", include_str!("../catalog/clock-digital.ps")),
    ("clock-ledsign", include_str!("../catalog/clock-ledsign.ps")),
    ("clock-morph", include_str!("../catalog/clock-morph.ps")),
    ("confetti", include_str!("../catalog/confetti.ps")),
    ("crawl", include_str!("../catalog/crawl.ps")),
    ("credits", include_str!("../catalog/credits.ps")),
    ("dotmatrix", include_str!("../catalog/dotmatrix.ps")),
    ("fire", include_str!("../catalog/fire.ps")),
    ("fireworks", include_str!("../catalog/fireworks.ps")),
    ("ledmatrix", include_str!("../catalog/ledmatrix.ps")),
    ("odometer", include_str!("../catalog/odometer.ps")),
    ("pipeline", include_str!("../catalog/pipeline.ps")),
    ("progress", include_str!("../catalog/progress.ps")),
    ("spinner", include_str!("../catalog/spinner.ps")),
    ("splitflap", include_str!("../catalog/splitflap.ps")),
    ("terminal", include_str!("../catalog/terminal.ps")),
    ("testdots", include_str!("../catalog/testdots.ps")),
    ("twiddle", include_str!("../catalog/twiddle.ps")),
];

/// The source of a catalog entry, by name.
pub fn source(name: &str) -> Option<&'static str> {
    CATALOG.iter().find(|e| e.0 == name).map(|e| e.1)
}

/// What an entry says about each of its parameters: the comment at the
/// end of the line that reads it (`/Style /ring param /style exch def  % /ring /dots`).
pub fn param_notes(source: &str) -> std::collections::BTreeMap<String, String> {
    let mut notes = std::collections::BTreeMap::new();
    for line in source.lines() {
        let line = line.trim_start();
        let Some((code, note)) = line.split_once('%') else {
            continue;
        };
        if !code.split_whitespace().any(|w| w == "param") {
            continue;
        }
        if let Some(name) = code
            .strip_prefix('/')
            .and_then(|r| r.split_whitespace().next())
        {
            notes.insert(name.to_string(), note.trim().to_string());
        }
    }
    notes
}

pub(crate) fn install(interp: &mut Interp, ctx: &Shared) {
    // /name .catknown bool
    reg(interp, ctx, ".catknown", |i, _| {
        let known = matches!(i.pop_value()?, Value::Name(n) if source(&n).is_some());
        i.push_bool(known);
        Ok(())
    });
    // /name .catsource (source)
    reg(interp, ctx, ".catsource", |i, ctx| {
        let name = match i.pop_value()? {
            Value::Name(n) => n,
            _ => return Err(PsError::Typecheck),
        };
        match source(&name) {
            Some(src) => {
                i.push_string(src.as_bytes());
                Ok(())
            }
            None => fail(
                ctx,
                PsError::Undefined(name.clone()),
                format!(
                    "the catalog has no entry /{name}. Entries: {}. `psanim catalog` lists them.",
                    CATALOG
                        .iter()
                        .map(|e| format!("/{}", e.0))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            ),
        }
    });
}
