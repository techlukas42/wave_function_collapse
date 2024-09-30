use array2d::Array2D;
use getset::Getters;
use itertools::Itertools;

#[derive(Clone, PartialEq, Eq, Hash, Debug, new, Getters)]
#[getset(get = "pub")]
pub struct Field {
    img_name: String,
    rotation: i32,
    sides: [String; 4],
    weight: u32,
}

#[derive(new, Getters)]
#[getset(get = "pub")]
pub struct Params<'p> {
    fields: &'p Vec<Field>,
    sides: &'p [Vec<Vec<&'p Field>>; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
pub struct Coord {
    x: usize,
    y: usize,
}

trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let mut already_seen = Vec::new();
        self.retain(|item| match already_seen.contains(item) {
            true => false,
            _ => {
                already_seen.push(item.clone());
                true
            }
        })
    }
}

pub fn update_field(params: &Params, wave: &mut Array2D<Vec<&Field>>, pos: Coord) -> usize {
    let mut next = find_real_neighbors(wave, pos);
    let mut updates: usize = 0;
    while let Some(pos) = next.pop() {
        let neighbors = find_neighbors(params, wave, pos.clone());
        if apply_constraints(wave.get_mut(pos.y, pos.x).unwrap(), &neighbors) {
            next.append(&mut find_real_neighbors(wave, pos));
            updates += 1;
            next.clear_duplicates();
        }
    }
    updates
}

fn apply_constraints(target: &mut Vec<&Field>, sides: &[Vec<&str>; 4]) -> bool {
    let old_len = target.len();
    target.retain(|field: &&Field| {
        field
            .sides()
            .iter()
            .zip(sides)
            .all(|(target_side, cmp_sides): (&String, &Vec<&str>)| {
                cmp_sides
                    .iter()
                    .any(|cmp_side: &&str| fits(target_side, cmp_side))
            })
    });
    // TODO save change
    target.len() != old_len
}

fn find_neighbors<'p>(
    params: &'p Params<'p>,
    wave: &Array2D<Vec<&'p Field>>,
    pos: Coord,
) -> [Vec<&'p str>; 4] {
    let mut offsets = [2, 3, 0, 1].iter();
    [
        {
            let y = pos.y as i32 - 1;
            if y < 0 {
                params.sides[0].get(pos.x).unwrap()
            } else {
                wave.get(y as usize, pos.x).unwrap()
            }
        },
        {
            let x = pos.x + 1;
            if x >= wave.column_len() {
                params.sides[1].get(pos.y).unwrap()
            } else {
                wave.get(pos.y, x).unwrap()
            }
        },
        {
            let y = pos.y + 1;
            if y >= wave.row_len() {
                params.sides[2].get(pos.x).unwrap()
            } else {
                wave.get(y, pos.x).unwrap()
            }
        },
        {
            let x = pos.x as i32 - 1;
            if x < 0 {
                params.sides[3].get(pos.x).unwrap()
            } else {
                wave.get(pos.y, x as usize).unwrap()
            }
        },
    ]
    .map(|side: &Vec<&Field>| {
        let offset = offsets.next().unwrap();
        let mut res = side.iter().map(|f| f.sides[*offset].as_str()).collect_vec();
        res.clear_duplicates();
        res
    })
}

fn find_real_neighbors(wave: &Array2D<Vec<&Field>>, pos: Coord) -> Vec<Coord> {
    let mut neighbors = Vec::with_capacity(4);
    if pos.y > 0 {
        neighbors.push(Coord::new(pos.x, pos.y - 1));
    }
    if pos.x < wave.row_len() - 1 {
        neighbors.push(Coord::new(pos.x + 1, pos.y));
    }
    if pos.y < wave.column_len() - 1 {
        neighbors.push(Coord::new(pos.x, pos.y + 1));
    }
    if pos.x > 0 {
        neighbors.push(Coord::new(pos.x - 1, pos.y));
    }
    neighbors
}

fn fits(a: &str, b: &str) -> bool {
    let av = a.split('-').collect_vec();
    let bv = b.split('-').collect_vec();

    if av.len() < 2 || bv.len() < 2 {
        return false;
    }
    let af = av.first().unwrap();
    let bf = bv.first().unwrap();
    let an = av.get(1).unwrap();
    let bn = bv.get(1).unwrap();
    let a_flag = av.get(2).unwrap_or(&"none");
    let b_flag = bv.get(2).unwrap_or(&"none");

    if a_flag.starts_with("u_") && b_flag.starts_with("u_") && a_flag == b_flag {
        return false;
    }

    if af == &"i" && bf == &"i" && an == bn {
        return true;
    }

    if ((af == &"q" && bf == &"p") || (af == &"p" && bf == &"q")) && an == bn {
        return true;
    }

    false
}

fn print_wave(wave: &Array2D<Vec<&Field>>) {
    wave.rows_iter()
        .for_each(|r| println!("{}", r.map(|f| entry_string(f)).join(", ")));
}

pub fn entry_string(entry: &Vec<&Field>) -> String {
    format!(
        "[{}]",
        entry.iter().map(|field| field.img_name.as_str()).join(", ")
    )
}

#[cfg(test)]
mod collapse_test;
