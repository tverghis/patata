mod coords;
pub use coords::DrawCoords;

const WIDTH_PIXELS: usize = 64;
const HEIGHT_PIXELS: usize = 32;

#[derive(Debug, Clone)]
pub struct Video {
    buffer: [u8; WIDTH_PIXELS * HEIGHT_PIXELS],
}

impl Default for Video {
    fn default() -> Self {
        Self {
            buffer: [0; WIDTH_PIXELS * HEIGHT_PIXELS],
        }
    }
}

impl Video {
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn draw(&mut self, sprite: &[u8], &DrawCoords { pos_x, pos_y }: &DrawCoords) -> bool {
        let mut has_overlap = false;

        for (row, &byte) in sprite.iter().enumerate() {
            for col in 0..8 {
                let sprite_pixel = byte & (0b1000_0000 >> col);
                let screen_pixel = &mut self.buffer[(pos_y + row) * WIDTH_PIXELS + (pos_x + col)];

                if sprite_pixel != 0 {
                    if *screen_pixel != 0 {
                        has_overlap = true;
                    }

                    *screen_pixel ^= 0xFF;
                }
            }
        }

        has_overlap
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clear() {
        let mut d = Video {
            buffer: [99; WIDTH_PIXELS * HEIGHT_PIXELS],
        };

        d.clear();

        assert!(d.buffer.iter().all(|i| *i == 0))
    }

    #[test]
    fn draw() {
        let sprite = [0xF0, 0x80, 0xF0, 0x80, 0x80];
        let coords = DrawCoords::new(0, 0);

        let mut video = Video::default();

        let has_overlap = video.draw(&sprite, &coords);

        assert!(!has_overlap);

        let mut expected_buffer = vec![
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00],
            vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00],
            vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ];

        expected_buffer
            .iter_mut()
            .for_each(|i| i.extend(vec![0; 56]));

        let expected: Vec<_> = expected_buffer.into_iter().flatten().collect();

        assert_eq!(
            expected,
            Vec::from(&video.buffer[0..(WIDTH_PIXELS * sprite.len())])
        );
    }

    #[test]
    fn draw_with_collision() {
        let f_sprite = [0xF0, 0x80, 0xF0, 0x80, 0x80];
        let e_sprite = [0xF0, 0x80, 0xF0, 0x80, 0xF0];

        let coords = DrawCoords::new(0, 0);

        let mut video = Video::default();

        let _ = video.draw(&f_sprite, &coords);
        let has_overlap = video.draw(&e_sprite, &coords);

        assert!(has_overlap);

        let mut expected_buffer = vec![
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            vec![0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00],
        ];

        expected_buffer
            .iter_mut()
            .for_each(|i| i.extend(vec![0; 56]));

        let expected: Vec<_> = expected_buffer.into_iter().flatten().collect();

        assert_eq!(
            expected,
            Vec::from(&video.buffer[0..(WIDTH_PIXELS * f_sprite.len())])
        );
    }
}
