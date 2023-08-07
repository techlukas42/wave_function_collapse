use array2d::Array2D;
use getset::Getters;
use itertools::Itertools;
use rand::rngs::SmallRng;
use rand::Rng;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq, Hash, Debug, new, Getters)]
#[getset(get = "pub")]
pub struct Field {
    img_name: String,
    rotation: i32,
    sides: [String; 4],
    weight: u32,
}

#[derive(new)]
pub struct Params<'p> {
    fields: Vec<Field>,
    sides: &'p [Vec<Vec<Field>>; 4],
}

#[derive(new, Debug)]
struct Change {
    old: Vec<Field>,
    chosen: bool,
    pos: Coord,
}

#[derive(PartialEq, Eq, new, Clone)]
struct Decision {
    previous_id: Option<usize>,
    pos: Coord,
    to: Field,
    id: usize,
}

impl Hash for Decision {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

struct DecisionTree {
    changes: Vec<Change>,
    decisions: Vec<Decision>,
    forbidden_decisions: HashSet<Decision>,
    id_counter: usize,
}

impl DecisionTree {
    fn new() -> DecisionTree {
        DecisionTree {
            changes: Vec::new(),
            decisions: Vec::new(),
            forbidden_decisions: HashSet::new(),
            id_counter: 0,
        }
    }

    fn decide(&mut self, wave: &mut Array2D<Vec<Field>>, pos: Coord, to: Field) {
        self.changes.push(Change::new(
            wave.get(pos.y, pos.x).unwrap().clone(),
            true,
            pos,
        ));
        let decision = Decision::new(
            self.decisions.last().map(|d| d.previous_id).unwrap_or(None),
            pos,
            to,
            self.id_counter,
        );
        self.id_counter += 1;
        self.decisions.push(decision);
    }

    fn forbid_last_decision(&mut self) -> Result<(), NotCollapsable> {
        if self.decisions.is_empty() {
            return Err(NotCollapsable);
        }
        self.forbidden_decisions.retain(|d| {
            d.previous_id != self.decisions.last().map(|f| f.previous_id).unwrap_or(None)
        });
        self.forbidden_decisions
            .insert(self.decisions.pop().unwrap());
        Ok(())
    }

    fn get_forbidden_options(&self, pos: Coord) -> Vec<Field> {
        self.forbidden_decisions
            .iter()
            .filter(|d| {
                d.previous_id == self.decisions.last().map(|f| f.previous_id).unwrap_or(None)
                    && d.pos == pos
            })
            .map(|d| d.to.clone())
            .collect_vec()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
struct Coord {
    x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotCollapsable;

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

/// creates a 2d array with a collapsed wave
/// the sides are compatible with the given entropy,
/// the entropy is read clockwise starting at the upper side of the top left corner,
/// but the entries are read according to their coordinates,
/// the size is determined by the size of the first (x) and second (y) vector,
/// if the other 2 vectors do not match the size everything above the size is discarded and
/// everything missing as full entropy,
/// coordinates start in the upper left corner
pub fn collapse_wave(
    params: &Params,
    rng: &mut SmallRng,
) -> Result<Array2D<Field>, NotCollapsable> {
    let x_size = params.sides[0].len();
    let y_size = params.sides[2].len();
    let mut wave = Array2D::filled_with(params.fields.clone(), y_size, x_size);
    let mut decision_tree = DecisionTree::new();

    apply_contrainst_sides(params, &mut wave, &mut decision_tree)?;
    while let Some(found) = find_lowest_entropy(params, rng, &wave, &decision_tree)? {
        let mut next = vec![found];
        let old = wave.get(found.y, found.x).unwrap().clone();
        let mut new = old.clone();
        new.retain(|field| {
            !decision_tree
                .get_forbidden_options(found)
                .iter()
                .any(|d| d == field)
        });
        let to = chose_field(rng, &new);
        decision_tree.decide(&mut wave, found, to.clone());
        _ = wave.set(found.y, found.x, vec![to]);
        if propagate(params, &mut wave, &mut decision_tree, &mut next).is_err() {
            undo(&mut wave, &mut decision_tree)?;
            next.clear();
        }
    }
    Ok(Array2D::from_iter_row_major(
        wave.elements_row_major_iter()
            .map(|vec| vec.first().unwrap().clone()),
        wave.row_len(),
        wave.column_len(),
    )
    .unwrap())
}

/// selects the coordinates (x, y) with the lowest entropy greater than one,
/// if multiple fields have the lowest entropy a random one is selected
fn find_lowest_entropy(
    params: &Params,
    rng: &mut SmallRng,
    wave: &Array2D<Vec<Field>>,
    decision_tree: &DecisionTree,
) -> Result<Option<Coord>, NotCollapsable> {
    let mut lowest_number = params.fields.len();
    let mut lowest_coord: Vec<Coord> = Vec::new();
    for cx in 0..wave.column_len() {
        for cy in 0..wave.row_len() {
            let c = wave.get(cy, cx).unwrap();
            if c.len() == lowest_number {
                lowest_coord.push(Coord::new(cx, cy));
            } else if c.len() > 1 && c.len() < lowest_number {
                lowest_coord.clear();
                lowest_coord.push(Coord::new(cx, cy));
                lowest_number = c.len();
            }
        }
    }
    if !lowest_coord.is_empty() {
        lowest_coord.retain(|coord| {
            let forbidden = decision_tree.get_forbidden_options(*coord);
            !wave
                .get(coord.y, coord.x)
                .unwrap()
                .iter()
                .all(|field| forbidden.iter().any(|decision| decision != field))
        });
        if lowest_coord.is_empty() {
            return Err(NotCollapsable);
        }
        return Ok(Some(
            *lowest_coord
                .get(rng.gen_range(0..lowest_coord.len()))
                .unwrap(),
        ));
    }

    Ok(None)
}

fn chose_field(rng: &mut SmallRng, fields: &[Field]) -> Field {
    let adjusted: Vec<Field> = fields
        .iter()
        .flat_map(|field| vec![field.clone(); field.weight as usize])
        .collect();
    adjusted
        .get(rng.gen_range(0..adjusted.len())) //TODO Change
        .expect("rng should generate within range")
        .clone()
}

fn apply_contrainst_sides(
    params: &Params,
    wave: &mut Array2D<Vec<Field>>,
    decision_tree: &mut DecisionTree,
) -> Result<(), NotCollapsable> {
    let mut next: Vec<Coord> = Vec::with_capacity(2 * wave.column_len() + 2 * wave.row_len());
    for x in 0..wave.column_len() {
        next.push(Coord::new(x, 0));
        next.push(Coord::new(x, wave.row_len() - 1));
    }

    for y in 1..(wave.row_len() - 1) {
        next.push(Coord::new(0, y));
        next.push(Coord::new(wave.column_len() - 1, y));
    }

    propagate(params, wave, decision_tree, &mut next)?;

    Ok(())
}

/// applies the contrainst to one field if it changes it returns all neighbors of that field
/// returns err if the application leaves a field empty
fn apply_contrainst_field(
    params: &Params,
    wave: &mut Array2D<Vec<Field>>,
    decision_tree: &mut DecisionTree,
    pos: Coord,
    mark_anyway: bool,
) -> Result<Vec<Coord>, NotCollapsable> {
    let outers = [
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
    ];
    let outer_sides = [
        outers[0]
            .iter()
            .map(|f| f.sides[2].as_str())
            .unique()
            .collect_vec(),
        outers[1]
            .iter()
            .map(|f| f.sides[3].as_str())
            .unique()
            .collect_vec(),
        outers[2]
            .iter()
            .map(|f| f.sides[0].as_str())
            .unique()
            .collect_vec(),
        outers[3]
            .iter()
            .map(|f| f.sides[1].as_str())
            .unique()
            .collect_vec(),
    ];

    let old = wave.get(pos.y, pos.x).unwrap();
    let old_c = old.clone();
    let old_len = old.len();

    let res = old
        .iter()
        .filter(|field| {
            let mut fit = true;
            for (i, side) in outer_sides.iter().enumerate() {
                fit &= side.iter().any(|out| fits(out, field.sides[i].as_str()));
            }
            fit
        })
        .cloned()
        .collect_vec();
    let res_len = res.len();

    if res_len == 0 {
        return Err(NotCollapsable);
    }

    wave.set(pos.y, pos.x, res).unwrap();

    let mut changed: Vec<Coord> = Vec::new();

    if res_len != old_len || mark_anyway {
        decision_tree.changes.push(Change {
            old: old_c.to_vec(),
            chosen: false,
            pos,
        });
        if pos.x > 0 {
            changed.push(Coord::new(pos.x - 1, pos.y));
        }
        if pos.y < wave.column_len() - 1 {
            changed.push(Coord::new(pos.x, pos.y + 1));
        }
        if pos.x < wave.row_len() - 1 {
            changed.push(Coord::new(pos.x + 1, pos.y));
        }
        if pos.y > 0 {
            changed.push(Coord::new(pos.x, pos.y - 1));
        }
    }

    Ok(changed)
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

fn propagate(
    params: &Params,
    wave: &mut Array2D<Vec<Field>>,
    decision_tree: &mut DecisionTree,
    next: &mut Vec<Coord>,
) -> Result<(), NotCollapsable> {
    if next.len() == 1 {
        let pos = next.pop().unwrap();
        next.append(&mut apply_contrainst_field(
            params,
            wave,
            decision_tree,
            pos,
            true,
        )?);
    }
    while !next.is_empty() {
        let pos = next.pop().unwrap();
        next.append(&mut apply_contrainst_field(
            params,
            wave,
            decision_tree,
            pos,
            false,
        )?);
        next.clear_duplicates();
    }
    Ok(())
}

fn undo(
    wave: &mut Array2D<Vec<Field>>,
    decision_tree: &mut DecisionTree,
) -> Result<(), NotCollapsable> {
    while let Some(cur) = decision_tree.changes.pop() {
        if cur.chosen {
            break;
        }
        _ = wave.set(cur.pos.y, cur.pos.x, cur.old);
    }
    decision_tree.forbid_last_decision()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use array2d::Array2D;
    use itertools::Itertools;
    use rand::{rngs::SmallRng, SeedableRng};

    use crate::collapse::{apply_contrainst_field, fits, Coord};

    use super::{apply_contrainst_sides, find_lowest_entropy, DecisionTree, Field, Params};

    fn gen_set() -> Vec<Field> {
        let substrate_string = String::from_str("substrate").unwrap();
        let i_substrate_string = String::from_str("i-substrate").unwrap();
        let wire_string = String::from_str("wire").unwrap();
        let i_wire_string = String::from_str("i-wire").unwrap();
        let track_string = String::from_str("track").unwrap();
        let i_track_string = String::from_str("i-track").unwrap();
        let cross_string = String::from_str("cross").unwrap();
        let substrate_sides = [
            i_substrate_string.clone(),
            i_substrate_string.clone(),
            i_substrate_string.clone(),
            i_substrate_string.clone(),
        ];
        let wire_sides = [
            i_wire_string.clone(),
            i_substrate_string.clone(),
            i_wire_string.clone(),
            i_substrate_string.clone(),
        ];
        let track_sides = [
            i_substrate_string.clone(),
            i_track_string.clone(),
            i_substrate_string.clone(),
            i_track_string.clone(),
        ];
        let cross_sides = [
            i_wire_string.clone(),
            i_track_string.clone(),
            i_wire_string.clone(),
            i_track_string.clone(),
        ];
        vec![
            Field::new(substrate_string, 0, substrate_sides, 1),
            Field::new(wire_string, 0, wire_sides, 1),
            Field::new(track_string, 0, track_sides, 1),
            Field::new(cross_string, 0, cross_sides, 1),
        ]
    }

    #[test]
    fn test_fits_identical_symmetrical() {
        assert!(fits("i-Substrate", "i-Substrate"));
    }

    #[test]
    fn test_fits_asymmetrical() {
        assert!(fits("q-Substrate", "p-Substrate"));
        assert!(fits("p-Substrate", "q-Substrate"));
    }

    #[test]
    fn test_not_fits_different_symmetrical() {
        assert!(!fits("i-Substrate", "i-Wire"));
    }

    #[test]
    fn test_not_fits_different_asymmetrical() {
        assert!(!fits("q-Substrate", "p-Wire"));
    }

    #[test]
    fn test_not_fits_identical_asymmetrical() {
        assert!(!fits("q-Substrate", "q-Substrate"));
        assert!(!fits("p-Substrate", "p-Substrate"));
    }

    #[test]
    fn test_lowest_entropy_found() {
        let set = gen_set();
        let sides = [
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ];
        let params = Params {
            fields: set.clone(),
            sides: &sides,
        };
        let wave = Array2D::from_columns(&vec![
            vec![
                set.clone(),
                set.clone(),
                set.iter()
                    .filter(|field| field.img_name() != &String::from_str("substrate").unwrap())
                    .map(|field| field.clone())
                    .collect_vec(),
            ],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ])
        .unwrap();

        let mut rng = SmallRng::seed_from_u64(5);

        let result = find_lowest_entropy(&params, &mut rng, &wave, &mut DecisionTree::new());

        assert_eq!(result, Ok(Some(Coord::new(0, 2))))
    }

    #[test]
    fn test_lowest_entropy_not_one() {
        let set = gen_set();
        let sides = [
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ];
        let params = Params {
            fields: set.clone(),
            sides: &sides,
        };
        let wave = Array2D::from_columns(&vec![
            vec![
                set.clone(),
                set.iter()
                    .filter(|field| field.img_name() == &String::from_str("substrate").unwrap())
                    .map(|field| field.clone())
                    .collect_vec(),
                set.iter()
                    .filter(|field| field.img_name() != &String::from_str("substrate").unwrap())
                    .map(|field| field.clone())
                    .collect_vec(),
            ],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ])
        .unwrap();

        let mut rng = SmallRng::seed_from_u64(5);

        let result = find_lowest_entropy(&params, &mut rng, &wave, &mut DecisionTree::new());

        assert_eq!(result, Ok(Some(Coord::new(0, 2))))
    }

    #[test]
    fn test_lowest_entropy_not_found() {
        let set = gen_set();
        let sides = [
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ];
        let short_set = set
            .iter()
            .filter(|field| field.img_name() == &String::from_str("substrate").unwrap())
            .map(|field| field.clone())
            .collect_vec();
        let params = Params {
            fields: set.clone(),
            sides: &sides,
        };
        let wave = Array2D::from_columns(&vec![
            vec![short_set.clone(), short_set.clone(), short_set.clone()],
            vec![short_set.clone(), short_set.clone(), short_set.clone()],
            vec![short_set.clone(), short_set.clone(), short_set.clone()],
        ])
        .unwrap();

        let mut rng = SmallRng::seed_from_u64(5);

        let result = find_lowest_entropy(&params, &mut rng, &wave, &mut DecisionTree::new());

        assert_eq!(result, Ok(None))
    }

    #[test]
    fn test_field_no_change_corner() {
        let set = gen_set();
        let sides = [
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ];
        let params = Params {
            fields: set.clone(),
            sides: &sides,
        };
        let mut wave = Array2D::from_columns(&vec![
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ])
        .unwrap();
        let res = apply_contrainst_field(
            &params,
            &mut wave,
            &mut DecisionTree::new(),
            Coord::new(0, 0),
            false,
        );
        assert_eq!(res, Ok(Vec::new()));
        assert_eq!(wave.get(0, 0).unwrap().len(), 4);
    }

    #[test]
    fn test_sides_no_change() {
        let set = gen_set();
        let sides = [
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ];
        let params = Params {
            fields: set.clone(),
            sides: &sides,
        };
        let mut wave = Array2D::from_columns(&vec![
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
            vec![set.clone(), set.clone(), set.clone()],
        ])
        .unwrap();
        let res = apply_contrainst_sides(&params, &mut wave, &mut DecisionTree::new());
        assert_eq!(res, Ok(()));
        assert_eq!(wave.get(0, 0).unwrap().len(), 4);
        assert_eq!(wave.get(0, 1).unwrap().len(), 4);
        assert_eq!(wave.get(0, 2).unwrap().len(), 4);
        assert_eq!(wave.get(1, 0).unwrap().len(), 4);
        assert_eq!(wave.get(1, 1).unwrap().len(), 4);
        assert_eq!(wave.get(1, 2).unwrap().len(), 4);
        assert_eq!(wave.get(2, 0).unwrap().len(), 4);
        assert_eq!(wave.get(2, 1).unwrap().len(), 4);
        assert_eq!(wave.get(2, 2).unwrap().len(), 4);
    }
}
