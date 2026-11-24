//! Turning a stream of frames into a file. This is the one part of psanim
//! that depends on the platform: video is encoded by an `ffmpeg` process
//! that reads raw frames on its standard input. PNG sequences need nothing
//! but the interpreter's own encoder.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

/// The container/codec an output path asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Mp4,
    WebM,
    Gif,
    /// A directory of `frame_00000.png`.
    PngSequence,
}

impl Format {
    /// Decide from the output path: its extension, or a directory for PNGs.
    pub fn from_path(path: &Path) -> Result<Format, String> {
        match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("mp4") | Some("m4v") | Some("mov") => Ok(Format::Mp4),
            Some("webm") => Ok(Format::WebM),
            Some("gif") => Ok(Format::Gif),
            None => Ok(Format::PngSequence),
            Some(other) => Err(format!(
                "cannot tell the output format from '.{other}': use .mp4, .webm, .gif, or a directory name for a PNG sequence"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Format::Mp4 => "mp4 (H.264, yuv420p)",
            Format::WebM => "webm (VP9, yuv420p)",
            Format::Gif => "gif",
            Format::PngSequence => "png sequence",
        }
    }
}

pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Where frames go.
pub enum Encoder {
    Ffmpeg { child: Child, stdin: Option<ChildStdin>, log: PathBuf },
    Pngs { dir: PathBuf, count: u64 },
}

impl Encoder {
    /// Start an encoder for `width`x`height` opaque RGBA frames at `fps`.
    /// `quality` is the codec's constant-quality setting (CRF): lower is
    /// better and larger; `None` picks a visually lossless default.
    pub fn start(
        format: Format,
        out: &Path,
        width: u32,
        height: u32,
        fps: f64,
        quality: Option<u32>,
        looping: bool,
    ) -> Result<Encoder, String> {
        if format == Format::PngSequence {
            std::fs::create_dir_all(out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
            return Ok(Encoder::Pngs { dir: out.to_path_buf(), count: 0 });
        }
        if !ffmpeg_available() {
            return Err(
                "ffmpeg was not found on PATH; install it to write video, or give a directory name to write a PNG sequence"
                    .into(),
            );
        }
        if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-loglevel", "error", "-y"])
            .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{width}x{height}")])
            .args(["-r", &format!("{fps}")])
            .args(["-i", "-", "-an"]);
        // yuv420p needs even dimensions; pad by a pixel when they are odd.
        let even = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
        match format {
            Format::Mp4 => {
                cmd.args(["-vf", even])
                    .args(["-c:v", "libx264", "-preset", "medium"])
                    .args(["-crf", &quality.unwrap_or(16).to_string()])
                    .args(["-pix_fmt", "yuv420p"])
                    // BT.709 tags so players do not guess; faststart so a
                    // browser can begin playing before the whole file arrives.
                    .args(["-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709"])
                    .args(["-movflags", "+faststart"]);
            }
            Format::WebM => {
                cmd.args(["-vf", even])
                    .args(["-c:v", "libvpx-vp9", "-b:v", "0"])
                    .args(["-crf", &quality.unwrap_or(28).to_string()])
                    .args(["-row-mt", "1", "-deadline", "good", "-cpu-used", "2"])
                    .args(["-pix_fmt", "yuv420p"]);
            }
            Format::Gif => {
                cmd.args([
                    "-vf",
                    "split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=sierra2_4a",
                ])
                .args(["-loop", if looping { "0" } else { "-1" }]);
            }
            Format::PngSequence => unreachable!("handled above"),
        }
        cmd.arg(out);
        let log = std::env::temp_dir().join(format!("psanim-ffmpeg-{}.log", std::process::id()));
        let log_file = std::fs::File::create(&log).map_err(|e| format!("cannot create a log file: {e}"))?;
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(log_file)
            .spawn()
            .map_err(|e| format!("cannot start ffmpeg: {e}"))?;
        let stdin = child.stdin.take();
        Ok(Encoder::Ffmpeg { child, stdin, log })
    }

    /// Add one frame. `rgba` is opaque RGBA for video; for a PNG sequence
    /// `png` is called to get the encoded file (which keeps transparency).
    pub fn write(&mut self, rgba: &[u8], png: impl FnOnce() -> Vec<u8>) -> Result<(), String> {
        match self {
            Encoder::Ffmpeg { stdin, log, .. } => {
                let pipe = stdin.as_mut().ok_or("the encoder has already been closed")?;
                pipe.write_all(rgba).map_err(|e| {
                    let said = std::fs::read_to_string(&*log).unwrap_or_default();
                    format!("ffmpeg stopped accepting frames ({e}): {}", said.trim())
                })
            }
            Encoder::Pngs { dir, count } => {
                let path = dir.join(format!("frame_{:05}.png", *count));
                *count += 1;
                std::fs::write(&path, png()).map_err(|e| format!("cannot write {}: {e}", path.display()))
            }
        }
    }

    /// Finish the file. For video this waits for ffmpeg to exit.
    pub fn finish(self) -> Result<(), String> {
        match self {
            Encoder::Ffmpeg { mut child, stdin, log } => {
                drop(stdin);
                let status = child.wait().map_err(|e| format!("waiting for ffmpeg: {e}"))?;
                let said = std::fs::read_to_string(&log).unwrap_or_default();
                let _ = std::fs::remove_file(&log);
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("ffmpeg failed ({status}): {}", said.trim()))
                }
            }
            Encoder::Pngs { .. } => Ok(()),
        }
    }
}
