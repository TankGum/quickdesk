//! Windows hands images over as a DIB (a BMP file without its file header).

/// A DIB (header + palette + pixels) as a complete BMP file the image crate reads.
pub(crate) fn dib_to_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    if dib.len() < 40 {
        return None;
    }
    let u32_at = |o: usize| u32::from_le_bytes([dib[o], dib[o + 1], dib[o + 2], dib[o + 3]]);
    let header = u32_at(0) as usize;
    let bit_count = u16::from_le_bytes([dib[14], dib[15]]) as usize;
    let compression = u32_at(16);
    let colors_used = u32_at(32) as usize;
    // BI_BITFIELDS with the plain 40-byte header: three masks follow it.
    let masks = if header == 40 && compression == 3 { 12 } else { 0 };
    let palette = if colors_used > 0 {
        colors_used * 4
    } else if bit_count <= 8 {
        (1 << bit_count) * 4
    } else {
        0
    };
    let offset = 14 + header + masks + palette;
    let mut bmp = Vec::with_capacity(14 + dib.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((14 + dib.len()) as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&(offset as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dib_becomes_a_readable_bmp() {
        // 2×1, 24-bit, bottom-up: one blue and one red pixel, rows padded to 4 bytes.
        let mut dib = Vec::new();
        dib.extend_from_slice(&40u32.to_le_bytes());
        dib.extend_from_slice(&2i32.to_le_bytes());
        dib.extend_from_slice(&1i32.to_le_bytes());
        dib.extend_from_slice(&1u16.to_le_bytes());
        dib.extend_from_slice(&24u16.to_le_bytes());
        dib.extend_from_slice(&[0u8; 24]);
        dib.extend_from_slice(&[255, 0, 0, 0, 0, 255, 0, 0]);
        let bmp = dib_to_bmp(&dib).unwrap();
        let img = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).unwrap().to_rgb8();
        assert_eq!(img.get_pixel(0, 0).0, [0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [255, 0, 0]);
    }
}
