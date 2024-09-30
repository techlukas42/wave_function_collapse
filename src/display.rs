use std::collections::HashMap;
use std::io;
use std::path::Path;

use array2d::Array2D;
use itertools::Itertools;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use crate::collapse::{entry_string, update_field, Coord, Field, Params};
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

pub fn interactive_render(set: Set, json: &Path) -> Result<(), String> {
    // sdl2 setup
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

    // wfc setup
    let x_size = 4;
    let y_size = 4;
    let base_vec = set.fields().iter().collect_vec();
    let mut side = Vec::with_capacity(32);
    for _ in 0..side.capacity() {
        side.push(base_vec.clone());
    }
    let sides = [side.clone(), side.clone(), side.clone(), side.clone()];
    let params = Params::new(&set.fields(), &sides);
    let mut wave: Array2D<Vec<&Field>> =
        Array2D::filled_with(set.fields().iter().collect_vec(), y_size, x_size);

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

        // Display
        for x in 0..wave.row_len() {
            for y in 0..wave.column_len() {
                let fields: &Vec<&Field> = wave.get(y, x).expect("coord should be in wave");
                let target = Rect::new(
                    (x as u32 * img_size) as i32,
                    (y as u32 * img_size) as i32,
                    img_size,
                    img_size,
                );
                for field in fields {
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
        }
        canvas.present();

        // input
        println!("Which field do you want to view? (x,y)");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        input.pop();
        input.pop();
        let mut split = input.split(",");
        let x: usize = split
            .next()
            .expect("invalid input")
            .parse()
            .expect("Not a number");
        let y: usize = split
            .next()
            .expect("invalid input")
            .parse()
            .expect("Not a number");
        println!(
            "field {}, {} is: {}",
            x,
            y,
            entry_string(wave.get(y, x).expect("Coord has to be in wave"))
        );
        println!("To which entry do you want to collapse it? (number)");
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        input.pop();
        input.pop();
        let n: usize = input.parse().expect("Not a number");
        wave.set(
            y,
            x,
            vec![wave.get(y, x).unwrap().get(n).expect("Outside field len")],
        )
        .expect("Outside wave");
        update_field(&params, &mut wave, Coord::new(x, y));
    }

    Ok(())
}
