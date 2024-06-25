mod arch;

pub use arch::*;

#[macro_export]
macro_rules! create_dist_dir {
  ($dir: expr) => {{
    let _ = std::fs::create_dir_all($dir).expect(format!("Can't create {:?} dir.", $dir).as_str());
  }};
}

#[macro_export]
macro_rules! check_and_clean_file_or_dir {
  ($dir: expr) => {
    if $dir.is_dir() {
      fs_extra::dir::remove(&$dir)
        .expect(format!("Remove {:?} folder failed.", &$dir.to_str().unwrap()).as_str());
    }
    if $dir.is_file() {
      fs_extra::file::remove(&$dir)
        .expect(format!("Remove {:?} file failed.", &$dir.to_str().unwrap()).as_str());
    }
  };
}

#[macro_export]
macro_rules! move_file {
  ($source: expr,$dist: expr) => {{
    // fix for symlink file
    let real_path = match $source.is_symlink() {
      true => {
        let symlink_path = $source.read_link().unwrap();
        let resolve_path = symlink_path
          .parent()
          .map_or_else(|| $source.clone(), |parent| parent.join($source))
          .canonicalize()
          .expect(format!("Get symlink {:?} real path failed.", &$source).as_str());
        resolve_path
      }
      false => $source.to_owned(),
    };
    if !real_path.is_file() {
      // TODO: print warning info
      return;
    }
    let mut options = fs_extra::file::CopyOptions::new();
    let real_file_name = real_path
      .file_name()
      .expect(format!("Get {:?} real file_name failed.", &real_path).as_str());
    let mut real_dist = $dist.clone();
    real_dist.set_file_name(real_file_name);
    // if exist will overwrite
    options = options.overwrite(true);
    fs_extra::file::copy(real_path, real_dist, &options)
      .expect(format!("Copy {:?} failed.", &$source).as_str());
  }};
}

#[macro_export]
macro_rules! create_project_file {
  ($strs: ident, $target: expr,$name: literal) => {{
    use std::io::prelude::*;
    #[cfg(not(target_os = "windows"))]
    use std::os::unix::fs::PermissionsExt;

    let mut tmp_file =
      std::fs::File::create($target).expect(format!("Create {} failed.", $name).as_str());
    // Windows don't need to set permissions
    // In another reason, we don't need to set permissions anymore, because we don't have any bash file.
    #[cfg(not(target_os = "windows"))]
    {
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
