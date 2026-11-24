//! Pixel operations on premultiplied RGBA8 buffers: the compositing a layer
//! needs, a Gaussian-like blur, temporal accumulation for motion blur, and
//! the conversions an encoder or a thumbnail wants.
//!
//! Buffers are `width * height * 4` bytes, rows top to bottom, each colour
//! channel already multiplied by alpha: the layout of
//! [`pscat::Page::premultiplied_rgba8`]. Nothing here depends on the
//! interpreter, a platform, or a clock.

/// How a layer's pixels combine with what is under them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    /// Source over destination.
    Normal,
    /// Add, clamped: light on light. Glows and sparks.
    Add,
    /// `1 - (1 - a)(1 - b)`: a gentler lightening than `Add`.
    Screen,
    /// `a * b`: darkening, like ink on paper.
    Multiply,
}

impl Blend {
    pub const NAMES: &'static [&'static str] = &["normal", "add", "screen", "multiply"];

    pub fn from_name(name: &str) -> Option<Blend> {
        Some(match name {
            "normal" => Blend::Normal,
            "add" => Blend::Add,
            "screen" => Blend::Screen,
            "multiply" => Blend::Multiply,
            _ => return None,
        })
    }
}

#[inline]
fn mul255(a: u32, b: u32) -> u32 {
    // Exact a*b/255 rounded, for a, b in 0..=255.
    let t = a * b + 128;
    (t + (t >> 8)) >> 8
}

/// Composite `src` onto `dst` with a uniform `opacity` (0..=1).
pub fn composite(dst: &mut [u8], src: &[u8], opacity: f64, blend: Blend) {
    debug_assert_eq!(dst.len(), src.len());
    let op = (opacity.clamp(0.0, 1.0) * 255.0).round() as u32;
    if op == 0 {
        return;
    }
    for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
        if s[3] == 0 && (blend != Blend::Add || (s[0] | s[1] | s[2]) == 0) {
            continue;
        }
        let sa = mul255(u32::from(s[3]), op);
        let sc = [
            mul255(u32::from(s[0]), op),
            mul255(u32::from(s[1]), op),
            mul255(u32::from(s[2]), op),
        ];
        let da = u32::from(d[3]);
        match blend {
            Blend::Normal => {
                let inv = 255 - sa;
                for c in 0..3 {
                    d[c] = (sc[c] + mul255(u32::from(d[c]), inv)).min(255) as u8;
                }
                d[3] = (sa + mul255(da, inv)).min(255) as u8;
            }
            Blend::Add => {
                for c in 0..3 {
                    d[c] = (sc[c] + u32::from(d[c])).min(255) as u8;
                }
                d[3] = (sa + da).min(255) as u8;
                // Keep the pixel premultiplied: no channel above alpha.
                let a = d[3];
                for c in &mut d[..3] {
                    *c = (*c).min(a);
                }
            }
            Blend::Screen => {
                for c in 0..3 {
                    let dc = u32::from(d[c]);
                    d[c] = (sc[c] + dc - mul255(sc[c], dc)).min(255) as u8;
                }
                d[3] = (sa + da - mul255(sa, da)).min(255) as u8;
            }
            Blend::Multiply => {
                // Premultiplied multiply: s*d + s*(1-da) + d*(1-sa).
                for c in 0..3 {
                    let dc = u32::from(d[c]);
                    d[c] = (mul255(sc[c], dc) + mul255(sc[c], 255 - da) + mul255(dc, 255 - sa))
                        .min(255) as u8;
                }
                d[3] = (sa + da - mul255(sa, da)).min(255) as u8;
            }
        }
    }
}

/// One box-blur pass along rows (`horizontal`) or columns.
fn box_pass(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize, horizontal: bool) {
    let (lines, len) = if horizontal { (h, w) } else { (w, h) };
    let at = |line: usize, i: usize| -> usize {
        if horizontal {
            (line * w + i) * 4
        } else {
            (i * w + line) * 4
        }
    };
    let win = (2 * r + 1) as u32;
    let half = win / 2;
    for line in 0..lines {
        let mut sum = [0u32; 4];
        // The window starts centred on pixel 0; edges repeat the border.
        for k in 0..=2 * r {
            let i = (k as isize - r as isize).clamp(0, len as isize - 1) as usize;
            let p = at(line, i);
            for c in 0..4 {
                sum[c] += u32::from(src[p + c]);
            }
        }
        for i in 0..len {
            let p = at(line, i);
            for c in 0..4 {
                dst[p + c] = ((sum[c] + half) / win) as u8;
            }
            let out = at(line, i.saturating_sub(r));
            let inn = at(line, (i + r + 1).min(len - 1));
            for c in 0..4 {
                sum[c] = sum[c] + u32::from(src[inn + c]) - u32::from(src[out + c]);
            }
        }
    }
}

/// The smallest rectangle holding every pixel that is not fully
/// transparent black, as `(x0, y0, x1, y1)` with exclusive ends.
fn ink_box(buf: &[u8], w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
    let row_has = |y: usize| buf[y * w * 4..(y + 1) * w * 4].iter().any(|b| *b != 0);
    let y0 = (0..h).find(|y| row_has(*y))?;
    let y1 = (y0..h).rev().find(|y| row_has(*y))? + 1;
    let (mut x0, mut x1) = (w, 0);
    for y in y0..y1 {
        let row = &buf[y * w * 4..(y + 1) * w * 4];
        if let Some(first) = row.chunks_exact(4).position(|p| p != [0, 0, 0, 0]) {
            x0 = x0.min(first);
            let last = row
                .chunks_exact(4)
                .rposition(|p| p != [0, 0, 0, 0])
                .unwrap_or(first);
            x1 = x1.max(last + 1);
        }
    }
    Some((x0, y0, x1, y1))
}

/// Blur `buf` in place with an approximately Gaussian kernel of standard
/// deviation `sigma` pixels: three box blurs, the classic approximation.
/// `scratch` is reused between calls and resized as needed.
///
/// Only the part of the image the blur can change is processed: the box
/// around what is drawn, grown by the kernel's reach. A layer with one
/// small bright thing on it costs a fraction of a full frame.
pub fn blur(buf: &mut [u8], w: usize, h: usize, sigma: f64, scratch: &mut Vec<u8>) {
    if !(sigma.is_finite() && sigma > 0.05) || w == 0 || h == 0 {
        return;
    }
    // Three boxes of radius r have variance r(r+1): pick r to match sigma^2.
    let ideal = ((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) / 2.0;
    let r = (ideal.round() as usize).clamp(1, w.max(h));
    let Some((x0, y0, x1, y1)) = ink_box(buf, w, h) else {
        return; // nothing drawn: nothing to blur
    };
    // Three passes each reach r pixels.
    let reach = 3 * r;
    let (x0, y0) = (x0.saturating_sub(reach), y0.saturating_sub(reach));
    let (x1, y1) = ((x1 + reach).min(w), (y1 + reach).min(h));
    let (cw, ch) = (x1 - x0, y1 - y0);
    if cw == w && ch == h {
        scratch.resize(buf.len(), 0);
        for _ in 0..3 {
            box_pass(buf, scratch, w, h, r, true);
            box_pass(scratch, buf, w, h, r, false);
        }
        return;
    }
    // Work on a copy of the region. Its edges are transparent (the region
    // was grown by the kernel's reach), so repeating the border there is
    // the same as the transparency beyond it; where the region meets the
    // image's own edge it repeats that edge, as the full blur does.
    let mut region = vec![0u8; cw * ch * 4];
    for y in 0..ch {
        let src = ((y0 + y) * w + x0) * 4;
        region[y * cw * 4..(y + 1) * cw * 4].copy_from_slice(&buf[src..src + cw * 4]);
    }
    scratch.resize(region.len(), 0);
    for _ in 0..3 {
        box_pass(&region, scratch, cw, ch, r, true);
        box_pass(scratch, &mut region, cw, ch, r, false);
    }
    for y in 0..ch {
        let dst = ((y0 + y) * w + x0) * 4;
        buf[dst..dst + cw * 4].copy_from_slice(&region[y * cw * 4..(y + 1) * cw * 4]);
    }
}

/// Add a blurred copy of `buf` back onto itself, `strength` times: a glow
/// around everything bright. `strength` 1 is a full-strength halo.
pub fn glow(buf: &mut [u8], w: usize, h: usize, sigma: f64, strength: f64, scratch: &mut Vec<u8>) {
    let mut halo = buf.to_vec();
    blur(&mut halo, w, h, sigma, scratch);
    composite(buf, &halo, strength, Blend::Add);
    if strength > 1.0 {
        composite(buf, &halo, strength - 1.0, Blend::Add);
    }
}

/// Sums frames to average them: motion blur by temporal supersampling.
#[derive(Default)]
pub struct Accumulator {
    sum: Vec<u32>,
    count: u32,
}

impl Accumulator {
    pub fn clear(&mut self, len: usize) {
        self.sum.clear();
        self.sum.resize(len, 0);
        self.count = 0;
    }

    pub fn add(&mut self, frame: &[u8]) {
        debug_assert_eq!(frame.len(), self.sum.len());
        for (s, p) in self.sum.iter_mut().zip(frame) {
            *s += u32::from(*p);
        }
        self.count += 1;
    }

    /// The average of the frames added, written into `out`.
    pub fn resolve(&self, out: &mut Vec<u8>) {
        let n = self.count.max(1);
        out.clear();
        out.extend(self.sum.iter().map(|s| ((s + n / 2) / n) as u8));
    }
}

/// Premultiplied pixels as straight (unassociated) RGBA.
pub fn to_straight(premul: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(premul.len());
    for p in premul.chunks_exact(4) {
        let a = u32::from(p[3]);
        if a == 255 || a == 0 {
            out.extend_from_slice(p);
        } else {
            let un = |c: u8| ((u32::from(c) * 255 + a / 2) / a).min(255) as u8;
            out.extend_from_slice(&[un(p[0]), un(p[1]), un(p[2]), p[3]]);
        }
    }
    out
}

/// Premultiplied pixels flattened onto an opaque `matte` colour, written
/// into `out` as opaque RGBA: what a video encoder is fed.
pub fn flatten_onto(premul: &[u8], matte: [u8; 3], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(premul.len());
    for p in premul.chunks_exact(4) {
        if p[3] == 255 {
            out.extend_from_slice(p);
        } else {
            let inv = 255 - u32::from(p[3]);
            out.extend_from_slice(&[
                (u32::from(p[0]) + mul255(u32::from(matte[0]), inv)).min(255) as u8,
                (u32::from(p[1]) + mul255(u32::from(matte[1]), inv)).min(255) as u8,
                (u32::from(p[2]) + mul255(u32::from(matte[2]), inv)).min(255) as u8,
                255,
            ]);
        }
    }
}

/// Scale an image down to `dw`x`dh` by averaging the source pixels each
/// destination pixel covers (a box filter; fine for thumbnails).
pub fn downscale(src: &[u8], sw: usize, sh: usize, dw: usize, dh: usize) -> Vec<u8> {
    let mut out = vec![0u8; dw * dh * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return out;
    }
    for dy in 0..dh {
        let y0 = dy * sh / dh;
        let y1 = (((dy + 1) * sh).div_ceil(dh)).clamp(y0 + 1, sh);
        for dx in 0..dw {
            let x0 = dx * sw / dw;
            let x1 = (((dx + 1) * sw).div_ceil(dw)).clamp(x0 + 1, sw);
            let mut sum = [0u32; 4];
            for y in y0..y1 {
                let row = &src[(y * sw + x0) * 4..(y * sw + x1) * 4];
                for p in row.chunks_exact(4) {
                    for c in 0..4 {
                        sum[c] += u32::from(p[c]);
                    }
                }
            }
            let n = ((y1 - y0) * (x1 - x0)) as u32;
            let o = (dy * dw + dx) * 4;
            for c in 0..4 {
                out[o + c] = ((sum[c] + n / 2) / n) as u8;
            }
        }
    }
    out
}

/// Summary statistics of a frame, cheap enough to compute for every frame:
/// what an agent checks before looking at pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    /// Mean of each straight colour channel over the frame, 0..=1, as seen
    /// over the matte.
    pub mean_rgb: [f64; 3],
    /// Mean luma (Rec. 709 weights on the gamma-encoded values), 0..=1.
    pub luma: f64,
    /// The fraction of pixels that differ from the frame's most common
    /// colour by more than a small threshold: how much of the frame is
    /// "something". Near 0 means a blank or flat frame.
    pub coverage: f64,
    /// Bounding box of those pixels, `[x0, y0, x1, y1]` in pixels with y
    /// down, or `None` when the frame is flat.
    pub bbox: Option<[u32; 4]>,
}

/// Compute [`Stats`] for an opaque RGBA frame (after [`flatten_onto`]).
pub fn stats(rgba: &[u8], w: usize, h: usize) -> Stats {
    let n = (w * h).max(1);
    // The most common colour, by a coarse 4-bit-per-channel histogram.
    let mut hist = vec![0u32; 4096];
    let mut sum = [0u64; 3];
    for p in rgba.chunks_exact(4) {
        hist[(usize::from(p[0] >> 4) << 8)
            | (usize::from(p[1] >> 4) << 4)
            | usize::from(p[2] >> 4)] += 1;
        for c in 0..3 {
            sum[c] += u64::from(p[c]);
        }
    }
    let mode = hist
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| **c)
        .map_or(0, |(i, _)| i);
    let (mut covered, mut bbox) = (0usize, None::<[u32; 4]>);
    for (i, p) in rgba.chunks_exact(4).enumerate() {
        let bin =
            (usize::from(p[0] >> 4) << 8) | (usize::from(p[1] >> 4) << 4) | usize::from(p[2] >> 4);
        if bin != mode {
            covered += 1;
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            bbox = Some(match bbox {
                None => [x, y, x + 1, y + 1],
                Some(b) => [b[0].min(x), b[1].min(y), b[2].max(x + 1), b[3].max(y + 1)],
            });
        }
    }
    let mean = |c: usize| sum[c] as f64 / (n as f64 * 255.0);
    Stats {
        mean_rgb: [mean(0), mean(1), mean(2)],
        luma: 0.2126 * mean(0) + 0.7152 * mean(1) + 0.0722 * mean(2),
        coverage: covered as f64 / n as f64,
        bbox,
    }
}

/// How different two frames are: the mean absolute difference per channel,
/// 0..=1. Zero for identical frames. Used to judge temporal continuity (a
/// spike between consecutive frames is a cut or a pop).
pub fn difference(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 1.0;
    }
    let total: u64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| u64::from(x.abs_diff(*y)))
        .sum();
    total as f64 / (a.len() as f64 * 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_composite_is_source_over() {
        let mut dst = vec![255, 0, 0, 255];
        composite(&mut dst, &[0, 0, 255, 255], 1.0, Blend::Normal);
        assert_eq!(dst, [0, 0, 255, 255]);
        let mut dst = vec![255, 0, 0, 255];
        composite(&mut dst, &[0, 0, 255, 255], 0.5, Blend::Normal);
        assert_eq!(dst, [127, 0, 128, 255]);
        // Onto transparent: the layer itself, scaled by opacity, premultiplied.
        let mut dst = vec![0, 0, 0, 0];
        composite(&mut dst, &[0, 200, 0, 200], 0.5, Blend::Normal);
        assert_eq!(dst, [0, 100, 0, 100]);
        // Opacity 0 leaves the destination alone.
        let mut dst = vec![1, 2, 3, 255];
        composite(&mut dst, &[9, 9, 9, 255], 0.0, Blend::Normal);
        assert_eq!(dst, [1, 2, 3, 255]);
    }

    #[test]
    fn blend_modes() {
        let mut d = vec![100, 100, 100, 255];
        composite(&mut d, &[200, 50, 0, 255], 1.0, Blend::Add);
        assert_eq!(d, [255, 150, 100, 255]);
        let mut d = vec![128, 128, 128, 255];
        composite(&mut d, &[128, 0, 255, 255], 1.0, Blend::Screen);
        assert_eq!(d, [192, 128, 255, 255]);
        let mut d = vec![128, 128, 128, 255];
        composite(&mut d, &[128, 0, 255, 255], 1.0, Blend::Multiply);
        assert_eq!(d, [64, 0, 128, 255]);
        // Multiply by a transparent source changes nothing.
        let mut d = vec![128, 128, 128, 255];
        composite(&mut d, &[0, 0, 0, 0], 1.0, Blend::Multiply);
        assert_eq!(d, [128, 128, 128, 255]);
        for name in Blend::NAMES {
            assert!(Blend::from_name(name).is_some());
        }
        assert!(Blend::from_name("dodge").is_none());
    }

    #[test]
    fn every_blend_keeps_pixels_premultiplied() {
        let vals = [0u8, 1, 64, 127, 128, 200, 255];
        for blend in [Blend::Normal, Blend::Add, Blend::Screen, Blend::Multiply] {
            for &da in &vals {
                for &sa in &vals {
                    for &op in &[0.3, 1.0] {
                        let mut d = vec![da, da / 2, 0, da];
                        composite(&mut d, &[sa, 0, sa / 3, sa], op, blend);
                        assert!(
                            d[0] <= d[3] && d[1] <= d[3] && d[2] <= d[3],
                            "{blend:?} {d:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn blur_preserves_flat_images_and_total_light() {
        let (w, h) = (32, 24);
        let mut flat = vec![77u8; w * h * 4];
        let mut scratch = Vec::new();
        blur(&mut flat, w, h, 3.0, &mut scratch);
        assert!(flat.iter().all(|b| *b == 77));

        let mut img = vec![0u8; w * h * 4];
        let centre = (12 * w + 16) * 4;
        img[centre..centre + 4].copy_from_slice(&[255, 255, 255, 255]);
        blur(&mut img, w, h, 2.0, &mut scratch);
        assert!(img[centre] < 60, "the spike spread: {}", img[centre]);
        assert!(
            img[centre] > img[centre + 4 * 3],
            "and still peaks at the centre"
        );
        // Symmetric left/right and up/down.
        assert_eq!(img[centre - 8], img[centre + 8]);
        assert_eq!(img[centre - 2 * w * 4], img[centre + 2 * w * 4]);
        // Sigma 0 is the identity.
        let before = img.clone();
        blur(&mut img, w, h, 0.0, &mut scratch);
        assert_eq!(img, before);
    }

    #[test]
    fn blurring_a_region_equals_blurring_the_whole_image() {
        // A small subject off-centre, and one touching an edge: the cropped
        // blur must give the bytes a full-frame blur gives.
        let (w, h) = (96, 64);
        for (sx, sy) in [(40usize, 30usize), (0, 10), (90, 60)] {
            let mut img = vec![0u8; w * h * 4];
            for y in sy..(sy + 5).min(h) {
                for x in sx..(sx + 6).min(w) {
                    img[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[200, 120, 40, 255]);
                }
            }
            let mut whole = img.clone();
            let mut scratch = Vec::new();
            let r = 2usize; // sigma 2.5 -> box radius 2
            whole_blur(&mut whole, w, h, r, &mut scratch);
            blur(&mut img, w, h, 2.5, &mut scratch);
            assert_eq!(img, whole, "subject at {sx},{sy}");
        }
        // An empty layer stays empty and costs nothing.
        let mut empty = vec![0u8; w * h * 4];
        blur(&mut empty, w, h, 4.0, &mut Vec::new());
        assert!(empty.iter().all(|b| *b == 0));
    }

    fn whole_blur(buf: &mut [u8], w: usize, h: usize, r: usize, scratch: &mut Vec<u8>) {
        scratch.resize(buf.len(), 0);
        for _ in 0..3 {
            box_pass(buf, scratch, w, h, r, true);
            box_pass(scratch, buf, w, h, r, false);
        }
    }

    #[test]
    fn accumulator_averages() {
        let mut acc = Accumulator::default();
        acc.clear(4);
        acc.add(&[0, 100, 255, 255]);
        acc.add(&[100, 100, 0, 255]);
        let mut out = Vec::new();
        acc.resolve(&mut out);
        assert_eq!(out, [50, 100, 128, 255]);
    }

    #[test]
    fn straight_and_flatten() {
        assert_eq!(to_straight(&[100, 50, 0, 200]), [128, 64, 0, 200]);
        assert_eq!(to_straight(&[9, 8, 7, 255]), [9, 8, 7, 255]);
        let mut out = Vec::new();
        flatten_onto(&[100, 0, 0, 100, 5, 6, 7, 255], [0, 0, 255], &mut out);
        assert_eq!(out, [100, 0, 155, 255, 5, 6, 7, 255]);
    }

    #[test]
    fn downscale_averages_blocks() {
        // 4x2 -> 2x1: left block black, right block white.
        let mut src = vec![0u8; 4 * 2 * 4];
        for y in 0..2 {
            for x in 2..4 {
                src[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4].copy_from_slice(&[255; 4]);
            }
        }
        for p in src.chunks_exact_mut(4) {
            p[3] = 255;
        }
        assert_eq!(
            downscale(&src, 4, 2, 2, 1),
            [0, 0, 0, 255, 255, 255, 255, 255]
        );
        // Non-integer ratio covers every source pixel at least once.
        let d = downscale(&src, 4, 2, 3, 1);
        assert_eq!(d.len(), 12);
        assert_eq!((d[0], d[8]), (0, 255));
    }

    #[test]
    fn stats_find_the_subject() {
        let (w, h) = (10, 10);
        let mut img = vec![0u8; w * h * 4];
        for p in img.chunks_exact_mut(4) {
            p.copy_from_slice(&[20, 20, 20, 255]);
        }
        for y in 2..5 {
            for x in 3..9 {
                img[(y * w + x) * 4..(y * w + x) * 4 + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let s = stats(&img, w, h);
        assert_eq!(s.bbox, Some([3, 2, 9, 5]));
        assert!((s.coverage - 0.18).abs() < 1e-9);
        let flat = stats(&vec![255u8; 400], 10, 10);
        assert_eq!((flat.coverage, flat.bbox), (0.0, None));
        assert!((flat.luma - 1.0).abs() < 1e-9);
    }

    #[test]
    fn difference_metric() {
        assert_eq!(difference(&[0, 0, 0, 255], &[0, 0, 0, 255]), 0.0);
        assert_eq!(difference(&[0, 0, 0, 0], &[255, 255, 255, 255]), 1.0);
        assert_eq!(difference(&[0], &[0, 0]), 1.0);
    }
}
