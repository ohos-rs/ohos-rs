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

  let artifact_parser = construct!(crate::ArtifactArgs { name, dist });
  construct!(crate::Options::Artifact(artifact_parser))
}
