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
}
