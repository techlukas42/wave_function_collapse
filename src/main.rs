extern crate sdl2;

#[macro_use]
extern crate derive_new;

use std::env;
use std::path::Path;

use display::interactive_render;

mod collapse;
mod console;
mod display;
mod parser;

fn run(set: &Path) -> Result<(), String> {
    let fields = parser::load(set);
    interactive_render(fields, set)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(|string| string.as_str())
        .unwrap_or("res\\circuit.json");
    run(Path::new(path))?;

    Ok(())
}
