// The jumpscare screen: when he reaches the player a picture fills the screen for under two seconds,
// flickering between two frames, shaking, then fading to black. This file is only the timing and the
// picture's file format; the drawing is `scare_draw.rs`. The picture itself is not in the source (it is
// someone else's artwork): `tools/prepare_jumpscare.py` makes it from your own copy, and without it the
// screen just flashes black.
//
// Picture file: width and height as two little-endian u32, then width * height RGBA bytes, two frames of
// equal height one above the other.

pub const DURATION_MS: u64 = 1_800;
/// How long each of the two frames stays up.
pub const FLICKER_MS: u64 = 70;
/// When the fade to black starts.
pub const FADE_FROM_MS: u64 = 1_300;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Image<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
}

/// Reads the picture file; None when it is cut short or its size is wrong.
pub fn parse(bytes: &[u8]) -> Option<Image<'_>> {
    let width = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?);
    let height = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?);
    let rgba = bytes.get(8..)?;
    let wanted = (width as usize).checked_mul(height as usize)?.checked_mul(4)?;
    if width == 0 || height == 0 || rgba.len() != wanted {
        return None;
    }
    Some(Image { width, height, rgba })
}

impl Image<'_> {
    /// The 1x1 stand-in the source ships with is not a picture.
    pub fn is_picture(&self) -> bool {
        self.width > 1 && self.height > 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    /// 0 to 9: which frame of the original animation shows (`jumpscare_1` to `jumpscare_10`).
    pub frame: usize,
    /// Offset in GUI pixels.
    pub shake: [f32; 2],
    /// 0 (none) to 1 (black).
    pub fade: f32,
}

/// What the screen shows `elapsed_ms` after the scare began; None once it is over.
/// The original's animation: five frames of him lunging at the camera, then a loop of the five after them.
pub fn frame_index(elapsed_ms: u64) -> usize {
    let step = (elapsed_ms / FLICKER_MS) as usize;
    if step < 5 { step } else { 5 + (step - 5) % 5 }
}

pub fn frame_at(elapsed_ms: u64) -> Option<Frame> {
    if elapsed_ms >= DURATION_MS {
        return None;
    }
    let t = elapsed_ms as f32;
    let fade = if elapsed_ms <= FADE_FROM_MS {
        0.0
    } else {
        (elapsed_ms - FADE_FROM_MS) as f32 / (DURATION_MS - FADE_FROM_MS) as f32
    };
    Some(Frame {
        frame: frame_index(elapsed_ms),
        shake: [(t * 0.9).sin() * 6.0, (t * 1.3).cos() * 4.0],
        fade,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(w: u32, h: u32, extra: usize) -> Vec<u8> {
        let mut v = w.to_le_bytes().to_vec();
        v.extend(h.to_le_bytes());
        v.extend(std::iter::repeat(7).take((w * h * 4) as usize + extra));
        v
    }

    #[test]
    fn reads_a_picture_and_refuses_a_bad_one() {
        let ok = file(3, 2, 0);
        let img = parse(&ok).unwrap();
        assert_eq!((img.width, img.height, img.rgba.len()), (3, 2, 24));
        assert!(parse(&file(3, 2, 1)).is_none());
        assert!(parse(&[1, 0, 0]).is_none());
        assert!(parse(&file(0, 5, 0)).is_none());
        assert!(parse(&[]).is_none());
        assert!(parse(&u32::MAX.to_le_bytes().iter().chain(&u32::MAX.to_le_bytes()).copied().collect::<Vec<_>>()).is_none());
    }

    #[test]
    fn the_stand_in_is_not_a_picture() {
        let stand_in = [1u8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 255];
        assert!(!parse(&stand_in).unwrap().is_picture());
        assert!(parse(&file(4, 4, 0)).unwrap().is_picture());
    }

    #[test]
    fn the_screen_flickers_then_fades_then_ends() {
        assert_eq!(frame_at(0).unwrap().frame, 0);
        assert_eq!(frame_at(FLICKER_MS).unwrap().frame, 1);
        assert_eq!(frame_at(4 * FLICKER_MS).unwrap().frame, 4);
        assert_eq!(frame_at(5 * FLICKER_MS).unwrap().frame, 5);
        assert_eq!(frame_at(10 * FLICKER_MS).unwrap().frame, 5);
        assert_eq!(frame_at(14 * FLICKER_MS).unwrap().frame, 9);
        assert_eq!(frame_at(FADE_FROM_MS).unwrap().fade, 0.0);
        let late = frame_at(DURATION_MS - 1).unwrap().fade;
        assert!(late > 0.99 && late <= 1.0, "{late}");
        assert!(frame_at(DURATION_MS).is_none());
        assert!(frame_at(u64::MAX).is_none());
    }

    #[test]
    fn the_shake_stays_small() {
        for ms in (0..DURATION_MS).step_by(7) {
            let f = frame_at(ms).unwrap();
            assert!(f.shake[0].abs() <= 6.0 && f.shake[1].abs() <= 4.0);
        }
    }
}
