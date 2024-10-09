use std::process::Command;

#[derive(Debug)]
pub struct GitConfig {
  pub author: String,
}

/// get current local git's config to init project
/// @author
pub fn get_git_config() -> GitConfig {
  let default_user = whoami::username();
  let output = Command::new("git").arg("config").arg("user.name").output();
  let username = match output {
    Ok(ref o) => String::from_utf8_lossy(&o.stdout),
    Err(_) => default_user.into(),
  };
  GitConfig {
    author: String::from(username).replace("\n", "").replace("\r", ""),
  }
}
