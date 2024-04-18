use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::PathBuf;

/// build file to har
pub fn generate_har(package_name: PathBuf, package_path: PathBuf) {
  let har = File::create(package_name).unwrap();
  let enc = GzEncoder::new(har, Compression::default());
  let mut tar = tar::Builder::new(enc);
  tar.append_dir_all("package", package_path).unwrap();
  tar.finish().unwrap();
}
