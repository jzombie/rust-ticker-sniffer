use std::fs::File;
use std::io::Write;

use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

pub fn run_build_utils() -> Result<(), Box<dyn std::error::Error>> {
    let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);

    let encoded: Vec<u8> = bincode::serialize(&world).unwrap();

    // Write the generated source code to a file
    let mut file = File::create("src/__dummy_generated__.bin")?;
    file.write_all(&encoded)?;

    Ok(())
}
