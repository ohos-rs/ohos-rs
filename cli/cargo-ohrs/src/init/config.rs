use git2::Config;

#[derive(Debug)]
struct GitConfig {
    pub author: String,
}

/// get current local git's config to init project
/// @author
pub fn get_git_config() -> GitConfig {
    let default_user = whoami::username();
    let config = Config::open_default().unwrap();
    let username = config.get_string("user.name").unwrap_or(default_user);
    GitConfig { author: username }
}

#[cfg(test)]
mod test {
    use crate::init::config::get_git_config;

    #[test]
    fn test_get_git_config() {
        let u = get_git_config();
        assert!(!u.author.is_empty());
    }
}