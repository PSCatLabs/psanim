//! Documentation that runs. Every `postscript` code fence in the skill and
//! the Markdown docs, every example and every gallery piece is loaded as a
//! movie and a spread of its frames rendered small. A recipe that stops
//! working fails here, not in front of a reader.
//!
//! A fence that starts with `%!PS` is a whole program. Any other fence is
//! the body of a scene on a 640x360 canvas, 4 seconds long. A fence marked
//! `postscript partial` is not run.

use std::path::{Path, PathBuf};

use psanim::{Movie, Options, Scale};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn files_under(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files_under(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
    out.sort();
}

/// The runnable fences of a Markdown file, with the line each starts on.
fn fences(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (n, line) in text.lines().enumerate() {
        let t = line.trim_start();
        match &mut current {
            None if t.starts_with("```postscript") && !t.contains("partial") => current = Some((n + 2, String::new())),
            None => {}
            Some(_) if t.starts_with("```") => out.extend(current.take()),
            Some((_, body)) => {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    out
}

/// Load `source` and render a few frames; the error names `what`.
fn exercise(what: &str, source: &str, dir: Option<&Path>) -> Result<(), String> {
    let options = Options {
        scale: Scale::Width(160),
        read_roots: Some(dir.map(|d| vec![d.to_path_buf()]).unwrap_or_default()),
        ..Options::default()
    };
    let mut movie = Movie::load(source, &options).map_err(|e| format!("{what}: {e}"))?;
    let frames = movie.meta().frames;
    let mut any_ink = false;
    // A pure movie is sampled across its length. One whose frames depend on
    // the ones before would have to render everything up to each sample, so
    // its opening second stands for it here (`scripts/build_gallery.sh`
    // renders the whole of every piece).
    let picks = if movie.meta().mode == psanim::Mode::Pure {
        psanim::sheet::spread(frames, 7)
    } else {
        (0..frames.min(24)).step_by(4).collect()
    };
    for k in picks {
        let f = movie.render_frame(k).map_err(|e| format!("{what}: {e}"))?;
        let first = &f.premultiplied[..4];
        any_ink |= f.premultiplied.chunks_exact(4).any(|p| p != first);
    }
    if !any_ink {
        return Err(format!("{what}: every sampled frame is one flat colour"));
    }
    Ok(())
}

#[test]
fn every_postscript_fence_in_the_docs_runs() {
    let root = root();
    let mut md = Vec::new();
    files_under(&root.join("skills"), "md", &mut md);
    files_under(&root.join("docs"), "md", &mut md);
    md.push(root.join("README.md"));
    let mut failures = Vec::new();
    let mut count = 0;
    for path in md.iter().filter(|p| p.exists()) {
        let text = std::fs::read_to_string(path).unwrap();
        for (line, body) in fences(&text) {
            count += 1;
            let source = if body.trim_start().starts_with("%!PS") {
                body
            } else {
                format!("<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.05 0.06 0.09] >> animation\n/scene {{\n{body}\n}} def\n")
            };
            let what = format!("{}:{line}", path.strip_prefix(&root).unwrap().display());
            if let Err(e) = exercise(&what, &source, None) {
                failures.push(e);
            }
        }
    }
    assert!(count >= 10, "only {count} fences found: is the skill where this test looks?");
    assert!(failures.is_empty(), "{} of {count} documentation examples fail:\n\n{}", failures.len(), failures.join("\n\n"));
}

#[test]
fn every_example_and_gallery_piece_loads_and_renders() {
    let root = root();
    let mut ps = Vec::new();
    files_under(&root.join("examples"), "ps", &mut ps);
    files_under(&root.join("gallery"), "ps", &mut ps);
    files_under(&root.join("skills"), "ps", &mut ps);
    assert!(!ps.is_empty());
    let mut failures = Vec::new();
    for path in &ps {
        let source = String::from_utf8_lossy(&std::fs::read(path).unwrap()).into_owned();
        let what = path.strip_prefix(&root).unwrap().display().to_string();
        if let Err(e) = exercise(&what, &source, path.parent()) {
            failures.push(e);
        }
    }
    assert!(failures.is_empty(), "{} of {} programs fail:\n\n{}", failures.len(), ps.len(), failures.join("\n\n"));
}
