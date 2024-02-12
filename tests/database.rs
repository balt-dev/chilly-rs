#![cfg(feature = "assets")]

use std::{
    collections::{BTreeSet, BTreeMap},
    path::PathBuf
};
use std::collections::HashMap;

use chilly::database::{
    structures::{Color, TileData, Tiling},
    Database
};

use std::process::ExitCode;

#[test]
// Do this so the error message prints pretty
fn main() -> ExitCode {
    match _main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Print the Display representation of the error
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_db = Database {
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
                layer: Some(255),
                ..Default::default()
            }),
            ("baz".into(), TileData {
                color: Color::Paletted { x: 2, y: 2 },
                sprite: "bazaz".into(),
                directory: "sample2".into(),
                tiling: Tiling::Character,
                author: "totally not balt".into(),
                ..Default::default()
            }),
            ("sample".into(), TileData {
                color: Color::Paletted { x: 2, y: 4 },
                sprite: "sample".into(),
                directory: "vanilla".into(),
                tiling: Tiling::Character,
                author: "Hempuli".into(),
                layer: Some(18),
                tile_index: Some((1, 0)),
                grid_index: Some((0, 1)),
                object_id: Some("object999".into()),
                tags: BTreeSet::from(["tag1".into(), "tag2".into()])
            }),
            ("sample2".into(), TileData {
                color: Color::Paletted { x: 3, y: 2 },
                sprite: "sample2".into(),
                directory: "vanilla".into(),
                tiling: Tiling::AutoTiled,
                author: "Hempuli".into(),
                layer: Some(16),
                tile_index: Some((1, 1)),
                grid_index: Some((0, 2)),
                object_id: Some("object950".into()),
                tags: BTreeSet::new()
            }),
            ("editor_sample".into(), TileData {
                color: Color::Paletted { x: 2, y: 3 },
                sprite: "ed_sprite".into(),
                directory: "vanilla".into(),
                tiling: Tiling::None,
                author: "Hempuli".into(),
                layer: Some(17),
                tags: BTreeSet::from(["tag1".into(), "tag2".into(), "tag3".into()]),
                ..Default::default()
            })
        ])
    };

    let mut database = Database::new();
    // Get testing assets directory
    let testing_path = PathBuf::from(file!());
    let custom_assets = testing_path.with_file_name("assets");
    let vanilla_assets = testing_path.with_file_name("notbaba");
    database.load_custom(custom_assets)?;
    database.load_vanilla(vanilla_assets)?;
    assert_eq!(database, sample_db);

    Ok(())
}
