use bpaf::{construct, long, Parser};

pub fn cli_artifact() -> impl Parser<crate::Options> {
  let dist = long("dist")
    .short('d')
    .argument::<String>("DIST")
    .help("Final product file path, the files in this path will be copied.")
    .fallback(String::from("dist"));

  let name = long("name")
    .short('n')
    .argument("NAME")
    .help(".har file product name.")
    .fallback(String::from("package"));

  let skip_libs = long("skip-libs")
    .help(
      "Do not copy the dynamic link library to the hap folder, will be set to false by default.",
    )
    .switch()
    .fallback(false);

  let package = long("package")
    .short('p')
    .help("Package to generate artifact for (workspace mode only). Can also be specified via cargo args: -- -p package")
    .argument::<String>("PACKAGE")
    .optional();

  let artifact_parser = construct!(crate::ArtifactArgs {
    name,
    dist,
    skip_libs,
    package,
  });
  construct!(crate::Options::Artifact(artifact_parser))
}
