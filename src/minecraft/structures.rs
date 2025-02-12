use crate::nbt_reader::nbt_reader::NbtReader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Cursor;

pub const PALETTE_SIGN_NAME: &str = "minecraft:oak_wall_sign";
pub const PALETTE_AIR_NAME: &str = "minecraft:air";

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftStructureSignFrontTextNbt {
    pub messages: [String; 4],
}

#[derive(Debug, Serialize, Deserialize)]
/// The content of the "nbt" property inside a sign block
pub struct MinecraftStructureSignNbt {
    pub front_text: MinecraftStructureSignFrontTextNbt,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftStructureBlockNbt {
    pub pos: [i32; 3],
    pub state: u32,
    pub nbt: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinecraftStructurePaletteNbt {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Refer to https://minecraft.wiki/w/Structure_file for the implementation format
pub struct MinecraftStructureNbt {
    pub size: [u32; 3],
    pub blocks: Vec<MinecraftStructureBlockNbt>,
    pub palette: Vec<MinecraftStructurePaletteNbt>,
}

pub fn read_minecraft_structure_file(path: &str) -> MinecraftStructureNbt {
    let reader = NbtReader::new();

    let data = reader.read_nbt_file(&path);
    let mut cursor = Cursor::new(data.unwrap());

    match reader.parse_nbt(&mut cursor) {
        Ok(value) => {
            let serialized = serde_json::to_string(&value).unwrap();
            let deserialized: MinecraftStructureNbt = serde_json::from_str(&serialized).unwrap();
            deserialized
        }
        Err(e) => panic!("Failed to read structure file {path}: {}", e),
    }
}
