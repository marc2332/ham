use std::fs;
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub enum Errors {
    Reading,
    Parsing,
}

pub struct Manifest {
    pub version: Option<String>,
}

impl Manifest {
    pub fn from_file(file: &str) -> Result<Self, Errors> {
        match fs::read_to_string(file) {
            Ok(content) => {
                let yaml = YamlLoader::load_from_str(content.as_str());

                if let Ok(yaml) = yaml {
                    // Manifest document
                    let manifest = yaml[0].clone();

                    // Manifest version
                    let version = manifest["version"]
                        .as_str()
                        .map(|version| version.to_string());

                    Ok(Manifest { version })
                } else {
                    Err(Errors::Parsing)
                }
            }
            Err(_) => Err(Errors::Reading),
        }
    }
}
