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

#[macro_export]
macro_rules! create_project_file {
  ($strs: ident, $target: expr,$name: literal) => {{
    use std::io::prelude::*;

    let mut tmp_file =
      std::fs::File::create($target).expect(format!("Create {} failed.", $name).as_str());


      #[cfg(target_family="unix")]
      {
      use std::os::fs::PermissionsExt;
      let mut perms = tmp_file.metadata().unwrap().permissions();
      perms.set_mode(0o755);

      tmp_file.set_permissions(perms).unwrap();
      }
    tmp_file
      .write_all($strs.as_bytes())
      .expect(format!("Write {} failed", $name).as_str());
    println!("Create {} succeed.", $name);
  }};
}
