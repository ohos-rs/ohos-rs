use owo_colors::OwoColorize;

pub fn render(is_support: bool) -> String {
  if is_support {
    return "✔".green().bold().to_string();
  }
  "✖".red().bold().to_string()
}
