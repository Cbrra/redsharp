use std::{
    fs::File,
    io::{self, Read},
};

pub fn read_file_code() -> Result<String, io::Error> {
    let mut file = File::open("main.redstone")?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}
