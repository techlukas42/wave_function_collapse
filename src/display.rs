use std::collections::HashMap;
use std::path::Path;

use array2d::Array2D;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use crate::collapse::Field;
use crate::parser::Set;

pub fn render(set: Set, wave: Array2D<Field>, json: &Path) -> Result<(), String> {
    let img_size: u32 = 14;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Wave Function Collapse", 448, 448)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let path = json
        .parent()
        .expect("json should be in a directroy")
        .join(set.dir());

    let texture_creator = canvas.texture_creator();
    let mut pngs = HashMap::with_capacity(set.fields().len());

    for field in set.fields() {
        pngs.insert(
            field.img_name(),
            texture_creator.load_texture(path.join(field.img_name()))?,
        );
    }

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }

        for x in 0..wave.row_len() {
            for y in 0..wave.column_len() {
                let field = wave.get(y, x).expect("coord should be in wave");
                let target = Rect::new(
                    (x as u32 * img_size) as i32,
                    (y as u32 * img_size) as i32,
                    img_size,
                    img_size,
                );
                let texture = pngs
                    .get(field.img_name())
                    .expect("wave should only produce names in the set");
                canvas.copy_ex(
                    texture,
                    None,
                    target,
                    *field.rotation() as f64,
                    None,
                    false,
                    false,
                )?;
            }
        }
        canvas.present();
    }

    Ok(())
}
