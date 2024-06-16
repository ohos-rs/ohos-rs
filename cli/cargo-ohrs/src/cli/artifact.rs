use bpaf::{construct, positional, short, Parser};

pub fn cli_artifact() -> impl Parser<crate::Options> {
  let name = positional::<String>("name").help("project name");

  let package_name = short('p')
    .long("package")
    .help("Ohpm package's name. If not set,will use project's name")
    .argument::<String>("PACKAGE_NAME")
    .optional()
    .catch();

  let init_parser = construct!(crate::InitArgs { package_name, name });
  construct!(crate::Options::Init(init_parser))
}
