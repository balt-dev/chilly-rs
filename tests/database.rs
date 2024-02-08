use std::{collections::HashMap, path::PathBuf};

use chilly::database::{
    structures::{Color, TileData, Tiling},
    Database
};

#[cfg(feature = "assets")]
use std::process::ExitCode;

#[cfg(feature = "assets")]
#[test]
// Do this so the error message prints pretty
fn main() -> ExitCode {
    match _main() {
        Ok(_) => return ExitCode::SUCCESS,
        Err(e) => {
            // Print the Display representation of the error
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    }
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new();
    // Get testing assets directory
    let testing_path = PathBuf::from(file!());
    let custom_assets = testing_path.with_file_name("assets");
    let vanilla_assets = testing_path.with_file_name("notbaba");
    database.load_custom(custom_assets)?;
    database.load_vanilla(vanilla_assets)?;
    assert_eq!(database, Database {
        tiles: HashMap::from([
            ("foo".into(), TileData {
                color: Color::Paletted { x: 0, y: 3 },
                sprite: "foosprite".into(),
                directory: "sample".into(),
                tiling: Tiling::None,
                author: "baltdev".into(),
                ..Default::default()
            }),
            ("bar".into(), TileData {
                color: Color::RGB { r: 255, g: 255, b: 128 },
                sprite: "barsprite".into(),
                directory: "sample".into(),
                tiling: Tiling::AnimDir,
                author: "baltdev".into(),
                tile_index: Some((0, 0)),
                object_id: Some("object999".into()),
                layer: Some(255)
            }),
            ("baz".into(), TileData {
                color: Color::Paletted { x: 2, y: 2 },
                sprite: "bazaz".into(),
                directory: "sample2".into(),
                tiling: Tiling::Character,
                author: "totally not balt".into(),
                ..Default::default()
            })
        ])
    });

    Ok(())
}