use std::fs::File;
use flate2::Compression;
use flate2::write::GzEncoder;

pub fn remove_har() {}

/// build file to har
pub fn generate_har () {
    let har = File::create("tmp.har").unwrap();
    let enc = GzEncoder::new(har,Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("","./").unwrap();
}