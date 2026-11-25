//! Command-line parsing: small, dependency-free, and strict. An unknown
//! flag is an error that names the flag, not something silently ignored.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use psanim::{Budget, Limits, MotionBlur, Options, Param, Scale, Signals};

/// Flags shared by every command that loads a movie.
#[derive(Default)]
pub struct Common {
    pub file: Option<PathBuf>,
    pub scale: Option<Scale>,
    pub fps: Option<f64>,
    pub duration: Option<f64>,
    pub seed: Option<i64>,
    pub params: BTreeMap<String, Param>,
    pub signal_files: Vec<PathBuf>,
    pub untrusted: bool,
    pub max_steps: Option<u64>,
    pub max_seconds: Option<f64>,
    pub max_memory_mb: Option<usize>,
    pub read_roots: Vec<PathBuf>,
    pub motion_blur: Option<u32>,
    pub shutter: Option<f64>,
    pub json: bool,
    pub quiet: bool,
    pub output: Option<PathBuf>,
    pub threads: usize,
    /// Flags a specific command understands, collected for it to read.
    pub extra: BTreeMap<String, String>,
}

/// Flags that take no value.
const SWITCHES: &[&str] = &[
    "libs",
    "json",
    "quiet",
    "untrusted",
    "markdown",
    "no-blur",
    "keep-frames",
    "all",
    "sheet",
];

impl Common {
    /// Parse `args` (after the command name). `known` lists the
    /// command-specific flags that are allowed.
    pub fn parse(args: &[String], known: &[&str]) -> Result<Common, String> {
        let mut c = Common::default();
        let mut it = args.iter().peekable();
        while let Some(arg) = it.next() {
            let Some(flag) = arg
                .strip_prefix("--")
                .or_else(|| arg.strip_prefix('-').filter(|f| f.len() == 1))
            else {
                if c.file.is_some() {
                    return Err(format!(
                        "unexpected argument '{arg}' (one program file per command)"
                    ));
                }
                c.file = Some(PathBuf::from(arg));
                continue;
            };
            let (flag, inline) = match flag.split_once('=') {
                Some((f, v)) => (f, Some(v.to_string())),
                None => (flag, None),
            };
            let flag = match flag {
                "o" => "output",
                "j" => "threads",
                "q" => "quiet",
                f => f,
            };
            let value =
                |it: &mut std::iter::Peekable<std::slice::Iter<String>>| -> Result<String, String> {
                    match inline.clone().or_else(|| it.next().cloned()) {
                        Some(v) => Ok(v),
                        None => Err(format!("--{flag} needs a value")),
                    }
                };
            fn num<T: std::str::FromStr>(flag: &str, v: &str) -> Result<T, String> {
                v.parse()
                    .map_err(|_| format!("--{flag}: '{v}' is not a valid number"))
            }
            match flag {
                "json" => c.json = true,
                "quiet" => c.quiet = true,
                "untrusted" => c.untrusted = true,
                "output" => c.output = Some(PathBuf::from(value(&mut it)?)),
                "scale" => c.scale = Some(Scale::Factor(num(flag, &value(&mut it)?)?)),
                "width" => c.scale = Some(Scale::Width(num(flag, &value(&mut it)?)?)),
                "height" => c.scale = Some(Scale::Height(num(flag, &value(&mut it)?)?)),
                "fps" => c.fps = Some(num(flag, &value(&mut it)?)?),
                "duration" => c.duration = Some(num(flag, &value(&mut it)?)?),
                "seed" => c.seed = Some(num(flag, &value(&mut it)?)?),
                "threads" => c.threads = num(flag, &value(&mut it)?)?,
                "max-steps" => c.max_steps = Some(num::<f64>(flag, &value(&mut it)?)? as u64),
                "max-seconds" => c.max_seconds = Some(num(flag, &value(&mut it)?)?),
                "max-memory-mb" => c.max_memory_mb = Some(num(flag, &value(&mut it)?)?),
                "motion-blur" => c.motion_blur = Some(num(flag, &value(&mut it)?)?),
                "shutter" => c.shutter = Some(num(flag, &value(&mut it)?)?),
                "read-root" => c.read_roots.push(PathBuf::from(value(&mut it)?)),
                "signals" => c.signal_files.push(PathBuf::from(value(&mut it)?)),
                "param" => {
                    let v = value(&mut it)?;
                    let (k, v) = v
                        .split_once('=')
                        .ok_or_else(|| format!("--param wants name=value, got '{v}'"))?;
                    c.params.insert(k.to_string(), Param::parse(v));
                }
                f if known.contains(&f) => {
                    let v = if SWITCHES.contains(&f) {
                        "true".to_string()
                    } else {
                        value(&mut it)?
                    };
                    c.extra.insert(f.to_string(), v);
                }
                f => {
                    return Err(format!("unknown option --{f} (see `psanim help`)"));
                }
            }
        }
        Ok(c)
    }

    pub fn extra_num<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>, String> {
        match self.extra.get(key) {
            None => Ok(None),
            Some(v) => v
                .parse()
                .map(Some)
                .map_err(|_| format!("--{key}: '{v}' is not a valid number")),
        }
    }

    pub fn has(&self, key: &str) -> bool {
        self.extra.contains_key(key)
    }

    /// The program file and its text.
    pub fn source(&self) -> Result<(PathBuf, String), String> {
        let path = self.file.clone().ok_or("no program file given")?;
        let bytes =
            std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        // PostScript is bytes; a movie program is text. Latin-1 bytes in
        // strings survive as the characters they are.
        let text = String::from_utf8(bytes)
            .unwrap_or_else(|e| e.into_bytes().iter().map(|b| *b as char).collect());
        Ok((path, text))
    }

    /// The engine options these flags ask for. `default_scale` applies when
    /// no size flag was given.
    pub fn options(&self, default_scale: Scale) -> Result<Options, String> {
        let mut signals = Signals::default();
        for f in &self.signal_files {
            let text = std::fs::read_to_string(f)
                .map_err(|e| format!("cannot read {}: {e}", f.display()))?;
            signals.extend(Signals::from_csv(&text).map_err(|e| format!("{}: {e}", f.display()))?);
        }
        let mut budget = if self.untrusted {
            Budget::untrusted()
        } else {
            Budget::trusted()
        };
        let cap = |mut l: Limits| {
            if let Some(s) = self.max_steps {
                l = l.with_max_steps(s);
            }
            if let Some(s) = self.max_seconds {
                l = l.with_max_time(std::time::Duration::from_secs_f64(s));
            }
            if let Some(mb) = self.max_memory_mb {
                l = l.with_max_memory(mb << 20);
            }
            l
        };
        budget.setup = cap(budget.setup);
        budget.frame = cap(budget.frame);
        if self.untrusted && self.max_memory_mb.is_none() {
            budget.setup = budget.setup.with_max_memory(1 << 30);
            budget.frame = budget.frame.with_max_memory(1 << 30);
        }
        // A program reads files beside itself, and nothing else, unless told
        // otherwise. An untrusted one reads only the built-in libraries.
        let read_roots = if !self.read_roots.is_empty() {
            Some(self.read_roots.clone())
        } else if self.untrusted {
            Some(Vec::new())
        } else {
            self.file.as_ref().and_then(|f| f.parent()).map(|d| {
                vec![if d.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    d.to_path_buf()
                }]
            })
        };
        let motion_blur = if self.has("no-blur") {
            Some(None)
        } else {
            match (self.motion_blur, self.shutter) {
                (Some(0), _) => Some(None),
                (Some(samples), shutter) => Some(Some(MotionBlur {
                    samples,
                    shutter: shutter.unwrap_or(0.5),
                })),
                (None, _) => None,
            }
        };
        Ok(Options {
            scale: self.scale.unwrap_or(default_scale),
            fps: self.fps,
            duration: self.duration,
            seed: self.seed,
            params: self.params.clone(),
            signals,
            budget,
            motion_blur,
            read_roots,
            base_dir: self
                .file
                .as_ref()
                .and_then(|f| f.parent())
                .filter(|d| !d.as_os_str().is_empty())
                .map(Path::to_path_buf),
            interrupt: None,
        })
    }
}

/// Parse a list like `0,1.5,3` or a single number.
pub fn number_list(text: &str, what: &str) -> Result<Vec<f64>, String> {
    text.split(',')
        .map(|p| {
            p.trim()
                .parse::<f64>()
                .map_err(|_| format!("{what}: '{p}' is not a number"))
        })
        .collect()
}
