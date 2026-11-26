//! Contact sheets: several frames of a movie on one labelled image, the
//! cheapest way to see motion without playing anything.
//!
//! The layout and lettering are PSCat's own review sheet
//! ([`pscat::sheet`]); this module picks the frames, scales them to
//! thumbnails, and captions each with its frame number and time.

use pscat::Page;
use pscat::sheet::{CellSize, RowSize, Sheet, SheetLayout, text_strip};

use crate::engine::Meta;
use crate::raster;

/// One thumbnail for a sheet: a frame already rendered.
pub struct Still {
    pub index: u64,
    pub time: f64,
    pub width: u32,
    pub height: u32,
    /// Premultiplied RGBA.
    pub premultiplied: Vec<u8>,
}

/// `count` frame indices spread evenly over a movie of `frames` frames,
/// first and last included.
///
/// ```
/// assert_eq!(psanim::sheet::spread(100, 5), [0, 25, 50, 74, 99]);
/// assert_eq!(psanim::sheet::spread(3, 8), [0, 1, 2]);
/// assert_eq!(psanim::sheet::spread(10, 1), [0]);
/// ```
pub fn spread(frames: u64, count: usize) -> Vec<u64> {
    if frames == 0 || count == 0 {
        return Vec::new();
    }
    if count as u64 >= frames {
        return (0..frames).collect();
    }
    if count == 1 {
        return vec![0];
    }
    (0..count)
        .map(|i| ((i as f64) * (frames - 1) as f64 / (count - 1) as f64).round() as u64)
        .collect()
}

/// Lay `stills` out in rows of `columns`, each scaled to `thumb_width`
/// pixels wide and captioned `#index  time`. `title` goes above the first
/// row. Transparent frames are shown over `matte`.
pub fn compose(
    stills: &[Still],
    columns: usize,
    thumb_width: u32,
    title: Option<&str>,
    matte: [u8; 3],
) -> Result<Page, String> {
    let captions: Vec<String> = stills
        .iter()
        .map(|s| format!("#{}  {:.2}s", s.index, s.time))
        .collect();
    compose_captioned(stills, &captions, columns, thumb_width, title, matte)
}

/// [`compose`] with a caption of your own under each still.
pub fn compose_captioned(
    stills: &[Still],
    captions: &[String],
    columns: usize,
    thumb_width: u32,
    title: Option<&str>,
    matte: [u8; 3],
) -> Result<Page, String> {
    if stills.is_empty() {
        return Err("a contact sheet needs at least one frame".into());
    }
    let columns = columns.clamp(1, stills.len());
    let err = |e: pscat::sheet::SheetError| e.to_string();
    let mut cells: Vec<(Page, Option<Page>)> = Vec::new();
    for (k, s) in stills.iter().enumerate() {
        let tw = thumb_width.clamp(16, s.width.max(16));
        let th = ((u64::from(s.height) * u64::from(tw)) as f64 / f64::from(s.width))
            .round()
            .max(1.0) as u32;
        let small = raster::downscale(
            &s.premultiplied,
            s.width as usize,
            s.height as usize,
            tw as usize,
            th as usize,
        );
        let mut flat = Vec::new();
        raster::flatten_onto(&small, matte, &mut flat);
        let page =
            Page::from_premultiplied_rgba8(tw, th, flat).ok_or("thumbnail size is not usable")?;
        let text = captions.get(k).map_or("", String::as_str);
        let caption = text_strip(text, 13, false).map_err(err)?;
        cells.push((page, caption));
    }
    let header = match title {
        Some(t) if !t.is_empty() => text_strip(t, 16, true).map_err(err)?,
        _ => None,
    };
    let size = |p: &Option<Page>| p.as_ref().map_or((0, 0), |p| (p.width(), p.height()));
    let rows: Vec<RowSize> = cells
        .chunks(columns)
        .enumerate()
        .map(|(r, row)| {
            RowSize::new(
                if r == 0 { size(&header) } else { (0, 0) },
                row.iter()
                    .map(|(img, cap)| CellSize::new((img.width(), img.height()), size(cap)))
                    .collect(),
            )
        })
        .collect();
    let layout = SheetLayout::new(&rows).map_err(err)?;
    let mut sheet = Sheet::new(&layout).map_err(err)?;
    if let Some(h) = &header {
        sheet.put_header(0, h).map_err(err)?;
    }
    for (k, (img, cap)) in cells.iter().enumerate() {
        let (r, c) = (k / columns, k % columns);
        sheet.put_image(r, c, img).map_err(err)?;
        if let Some(cap) = cap {
            sheet.put_caption(r, c, cap).map_err(err)?;
        }
    }
    Ok(sheet.into_page())
}

/// The title line a sheet gets by default: the movie's title and its vital
/// statistics.
pub fn default_title(meta: &Meta, name: &str) -> String {
    format!(
        "{}  {}x{}  {:.2}s @ {} fps  {} frames  {}",
        meta.title.as_deref().unwrap_or(name),
        meta.width,
        meta.height,
        meta.duration,
        meta.fps,
        meta.frames,
        meta.mode.name()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn still(index: u64, shade: u8) -> Still {
        Still {
            index,
            time: index as f64 / 10.0,
            width: 80,
            height: 40,
            premultiplied: [shade, shade, shade, 255].repeat(80 * 40),
        }
    }

    #[test]
    fn spread_covers_both_ends() {
        assert_eq!(spread(100, 5), [0, 25, 50, 74, 99]);
        assert_eq!(spread(2, 2), [0, 1]);
        assert!(spread(0, 4).is_empty());
        assert!(spread(9, 0).is_empty());
        let s = spread(1000, 24);
        assert_eq!((s[0], s[23], s.len()), (0, 999, 24));
        assert!(s.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn sheet_holds_every_thumbnail() {
        let stills: Vec<Still> = (0..6).map(|k| still(k * 3, 40 * k as u8 + 20)).collect();
        let page = compose(&stills, 3, 40, Some("Test movie"), [0, 0, 0]).unwrap();
        // Two rows of three 40x20 thumbnails, plus captions, gaps and a title.
        assert!(
            page.width() >= 120 && page.height() >= 40 + 2 * 13,
            "{}x{}",
            page.width(),
            page.height()
        );
        let px = page.rgba8();
        // Every thumbnail's grey appears somewhere on the sheet.
        for k in 0..6u8 {
            let shade = 40 * k + 20;
            assert!(
                px.chunks(4)
                    .any(|p| p[0] == shade && p[1] == shade && p[2] == shade),
                "shade {shade}"
            );
        }
        assert!(compose(&[], 3, 40, None, [0, 0, 0]).is_err());
        // One column, no title.
        let tall = compose(&stills[..2], 1, 40, None, [0, 0, 0]).unwrap();
        assert!(tall.height() > tall.width());
    }
}
