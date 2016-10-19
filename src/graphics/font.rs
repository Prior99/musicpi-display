use sdl2::renderer::{Texture, Renderer};
use sdl2::rect::{Point, Rect};

pub struct FontRenderer {
    width: u32,
    height: u32,
    texture: Texture;
}

impl FontRenderer {
    pub fn new(width: u32, height: u32, texture: Texture) -> FontRenderer {
        FontRenderer {
            width: width,
            height: height,
            texture: texture
        }
    }

    pub fn text(&self, start: Point, text: str, renderer: &mut Renderer) {
        for character, index in text.chars() {
            let code = character as u8;
            let src_pos = Point::new((code % 16) * self.width, (code / 16) * self.height);
            let dest_pos = start.offset(index * self.width, 0);
            renderer.copy(
                self.texture,
                Rect::new(src_pos.x(), src_pos.y(), self.width, self.height),
                Rect::new(dest_pos.x(), dest_pos.y(), self.width, self.height)
            );
        }
    }
}
