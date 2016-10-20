use sdl2::render::{Texture, Renderer};
use sdl2::rect::{Point, Rect};

pub struct FontRenderer {
    pub width: u32,
    pub height: u32,
    pub texture: Texture
}

impl FontRenderer {
    pub fn new(width: u32, height: u32, texture: Texture) -> FontRenderer {
        FontRenderer {
            width: width,
            height: height,
            texture: texture
        }
    }

    pub fn text(&self, start: Point, text: &str, renderer: &mut Renderer) {
        for (index, character) in text.chars().enumerate() {
            let code = character as u8;
            let src_pos = Point::new((code as i32 % 16) * self.width as i32, (code as i32 / 16) * self.height as i32);
            let dest_pos = start.offset((index as u32 * (self.width + 1)) as i32, 0);
            renderer.copy(
                &self.texture,
                Some(Rect::new(src_pos.x(), src_pos.y(), self.width, self.height)),
                Some(Rect::new(dest_pos.x(), dest_pos.y(), self.width, self.height))
            );
        }
    }
}
