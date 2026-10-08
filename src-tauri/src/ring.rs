//! Renders the top-bar usage ring: an anti-aliased arc that fills clockwise
//! from 12 o'clock, coloured by how close the limit is.

pub const SIZE: u32 = 64;

type Rgba = [u8; 4];

const TRACK: Rgba = [255, 255, 255, 70];
const NO_DATA: Rgba = [160, 160, 160, 200];

pub fn color_for(percent: f64) -> Rgba {
    if percent >= 85.0 {
        [239, 68, 68, 255] // red
    } else if percent >= 60.0 {
        [245, 158, 11, 255] // amber
    } else {
        [34, 197, 94, 255] // green
    }
}

/// RGBA pixels (SIZE × SIZE). `None` draws an empty grey ring.
pub fn render(percent: Option<f64>) -> Vec<u8> {
    render_sized(percent, SIZE)
}

/// RGBA pixels of a `side × side` ring.
pub fn render_sized(percent: Option<f64>, side: u32) -> Vec<u8> {
    let size = side as f64;
    let center = size / 2.0;
    let outer = size / 2.0 - (size / 32.0).max(1.0);
    let inner = outer - size * 0.2;
    let fill = percent.map(|p| p.clamp(0.0, 100.0) / 100.0);
    let color = percent.map(color_for).unwrap_or(NO_DATA);

    let mut px = vec![0u8; (side * side * 4) as usize];
    for y in 0..side {
        for x in 0..side {
            let (dx, dy) = (x as f64 + 0.5 - center, y as f64 + 0.5 - center);
            let r = (dx * dx + dy * dy).sqrt();
            // Coverage of the ring band with a 1px soft edge on both sides.
            let coverage = (r - inner + 0.5).clamp(0.0, 1.0) * (outer - r + 0.5).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }
            // Angle from 12 o'clock, clockwise, in [0, 1).
            let turn = (dx.atan2(-dy) / std::f64::consts::TAU).rem_euclid(1.0);
            let base = match fill {
                Some(f) if turn < f => color,
                Some(_) => TRACK,
                None => NO_DATA,
            };
            let i = ((y * side + x) * 4) as usize;
            px[i..i + 3].copy_from_slice(&base[..3]);
            px[i + 3] = (base[3] as f64 * coverage).round() as u8;
        }
    }
    px
}

/// QuickDesk's tray glyph: three rounded bars (as on the app icon) in white,
/// so it sits with GNOME's monochrome status icons. RGBA, `side × side`.
pub fn app_glyph(side: u32) -> Vec<u8> {
    let s = side as f64;
    let (left, bar, gap) = (s * 0.16, s * 0.15, s * 0.12);
    let top = (s - 3.0 * bar - 2.0 * gap) / 2.0;
    // (x0, x1, y0) of each bar; the middle one is shorter.
    let bars = [(left, s - left, top), (left, s * 0.66, top + bar + gap), (left, s - left, top + 2.0 * (bar + gap))];
    let inside = |x: f64, y: f64| {
        bars.iter().any(|&(x0, x1, y0)| {
            // Rounded rectangle with fully round ends.
            let r = bar / 2.0;
            let cy = y0 + r;
            let cx = x.clamp(x0 + r, x1 - r);
            (x - cx).powi(2) + (y - cy).powi(2) <= r * r
        })
    };
    const SS: u32 = 4; // supersampling per axis, for smooth edges
    let mut px = vec![0u8; (side * side * 4) as usize];
    for y in 0..side {
        for x in 0..side {
            let hits = (0..SS * SS)
                .filter(|k| {
                    let (sx, sy) = ((k % SS) as f64 + 0.5, (k / SS) as f64 + 0.5);
                    inside(x as f64 + sx / SS as f64, y as f64 + sy / SS as f64)
                })
                .count();
            if hits > 0 {
                let i = ((y * side + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[255, 255, 255, (255 * hits / (SS * SS) as usize) as u8]);
            }
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(px: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    }

    #[test]
    fn fills_clockwise_from_top_with_threshold_colours() {
        let px = render(Some(30.0));
        assert_eq!(px.len(), (SIZE * SIZE * 4) as usize);
        // A row inside the ring near the top: just right of 12 o'clock is filled,
        // just left of it (near 100%) is track.
        let band_y = 6;
        assert_eq!(&pixel(&px, SIZE / 2 + 2, band_y)[..3], &[34, 197, 94]);
        assert_eq!(pixel(&px, SIZE / 2 - 3, band_y), TRACK);
        // Centre and corners stay transparent.
        assert_eq!(pixel(&px, SIZE / 2, SIZE / 2)[3], 0);
        assert_eq!(pixel(&px, 0, 0)[3], 0);
        assert_eq!(color_for(70.0), [245, 158, 11, 255]);
        assert_eq!(color_for(90.0), [239, 68, 68, 255]);
        assert_eq!(&pixel(&render(None), SIZE / 2, band_y)[..3], &NO_DATA[..3]);
    }

    #[test]
    fn glyph_is_white_bars_on_transparent() {
        let px = app_glyph(SIZE);
        // Middle of the top bar is solid white; the corners and the gap
        // right of the short middle bar are empty.
        assert_eq!(pixel(&px, SIZE / 2, SIZE / 2 - SIZE / 4 - 1), [255, 255, 255, 255]);
        assert_eq!(pixel(&px, 0, 0)[3], 0);
        assert_eq!(pixel(&px, SIZE * 3 / 4, SIZE / 2)[3], 0);
        assert_eq!(pixel(&px, SIZE / 3, SIZE / 2), [255, 255, 255, 255]);
    }
}
