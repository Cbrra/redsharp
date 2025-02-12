use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Wrapper to create and write the datapack
pub struct Datapack {
    world_datapack_path: Option<PathBuf>,
}

impl Datapack {
    pub fn new(world_path: Option<PathBuf>) -> Self {
        Self {
            world_datapack_path: world_path.map(|p| p.join("datapacks\\redsharp")),
        }
    }

    pub fn write_generate(&self) {
        let instructions = vec![
            "function redsharp:nodes",
            "function redsharp:edges"
        ];

        self.write_file(
            Path::new(".\\redsharp\\data\\redsharp\\function\\generate.mcfunction").to_path_buf(),
            instructions.join("\n").as_bytes(),
        )
        .expect("Failed to write datapack generate file");
    }

    pub fn write_nodes(&self, instructions: &Vec<String>) {
        self.write_file(
            Path::new(".\\redsharp\\data\\redsharp\\function\\nodes.mcfunction").to_path_buf(),
            instructions.join("\n").as_bytes(),
        )
        .expect("Failed to write datapack nodes file");
    }

    pub fn write_edges(&self, instructions: &Vec<String>) {
        self.write_file(
            Path::new(".\\redsharp\\data\\redsharp\\function\\edges.mcfunction").to_path_buf(),
            instructions.join("\n").as_bytes(),
        )
        .expect("Failed to write datapack edges file");
    }

    pub fn write_datapack(&self) {
        if let Some(path) = &self.world_datapack_path {
            let src = Path::new(".\\redsharp");
            self.copy_dir_all(src, path)
                .expect("Failed to copy the datapack to the world folder");
        }
    }

    fn write_file(&self, path: PathBuf, content: &[u8]) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write(content)?;
        Ok(())
    }

    fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                self.copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }

        Ok(())
    }
}
