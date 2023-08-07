use getset::Getters;
use std::{fs, path::Path};

use crate::collapse::Field;
use serde::Deserialize;

#[derive(Deserialize)]
struct Data {
    name: String,
    rotateable: bool,
    sides: Vec<String>,
    weight: u32,
}

impl Data {
    fn to_field(&self) -> Vec<Field> {
        let sides = [
            self.sides.get(0).expect("missing side").to_string(),
            self.sides.get(1).expect("missing side").to_string(),
            self.sides.get(2).expect("missing side").to_string(),
            self.sides.get(3).expect("missing side").to_string(),
        ];
        if self.rotateable {
            let mut sides1 = sides.clone();
            sides1.rotate_right(1);
            let mut sides2 = sides.clone();
            sides2.rotate_right(2);
            let mut sides3 = sides.clone();
            sides3.rotate_right(3);
            vec![
                Field::new(self.name.clone(), 0, sides, self.weight),
                Field::new(self.name.clone(), 90, sides1, self.weight),
                Field::new(self.name.clone(), 180, sides2, self.weight),
                Field::new(self.name.clone(), 270, sides3, self.weight),
            ]
        } else {
            vec![Field::new(self.name.clone(), 0, sides, self.weight)]
        }
    }
}

#[derive(Deserialize)]
struct DataSet {
    dir: String,
    fields: Vec<Data>,
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Set {
    dir: String,
    fields: Vec<Field>,
}

impl DataSet {
    fn to_set(&self) -> Set {
        let mut fields: Vec<Field> = Vec::new();
        self.fields
            .iter()
            .for_each(|data| fields.append(&mut data.to_field()));
        Set {
            dir: self.dir.clone(),
            fields,
        }
    }
}

pub fn load(set: &Path) -> Set {
    let contents = fs::read_to_string(set).expect("Couldn't find or load the set file");
    let set_data: DataSet =
        serde_json::from_str(contents.as_str()).expect("Couldn't parse set file");
    set_data.to_set()
}
