use std::{fs, io::Read, path::PathBuf};

pub mod audio_player;

fn file_as_raw_bytes(path: PathBuf) -> Vec<u8> {
    let mut bytes = Vec::new();
    fs::File::open(path.clone())
        .expect("error opening file")
        .read_to_end(&mut bytes)
        .expect("error reading file");
    bytes
}
