extern crate sdl2;

#[macro_use]
extern crate derive_new;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::Path;

use crate::collapse::Params;

mod collapse;
mod display;
mod parser;

fn run(set: &Path, seed: u64) -> Result<(), String> {
    let fields = parser::load(set);
    let base_vec = vec![fields.fields().first().expect("loaded empty set").clone()];
    let mut side = Vec::with_capacity(32);
    for _ in 0..side.capacity() {
        side.push(base_vec.clone());
    }

    let sides = [side.clone(), side.clone(), side.clone(), side.clone()];
    let params = Params::new(fields.fields().clone(), &sides);
    let mut rng = SmallRng::seed_from_u64(seed);

    let collapsed_wave = collapse::collapse_wave(&params, &mut rng).expect("wave not collapsed");
    display::render(fields, collapsed_wave, set)
}

fn main() -> Result<(), String> {
    run(Path::new("res\\circuit.json"), 7)?;

    Ok(())
}
