extern crate sdl2;

#[macro_use]
extern crate derive_new;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::env;
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
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(|string| string.as_str())
        .unwrap_or("res\\circuit.json");
    let seed = args
        .get(2)
        .map(|str| str.parse::<u64>().expect("seed must be a valid number"))
        .unwrap_or(0);
    run(Path::new(path), seed)?;

    Ok(())
}
