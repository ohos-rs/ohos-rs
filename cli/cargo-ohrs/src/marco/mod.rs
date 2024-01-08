#[macro_export]
macro_rules! create_dist_dir {
  ($dir: expr) => {{
    let _ = std::fs::create_dir_all($dir).expect(format!("Can't create {:?} dir.", $dir).as_str());
  }};
}

#[macro_export]
macro_rules! move_file {
  ($source: expr,$dist: expr) => {{
    let mut options = fs_extra::file::CopyOptions::new();
    // if exist will overwrite
    options = options.overwrite(true);
    fs_extra::file::move_file($source, $dist, &options).unwrap();
  }};
}
