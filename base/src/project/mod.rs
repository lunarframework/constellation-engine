use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct Project {
    pub path: PathBuf,
    pub config: Config,
}

impl Project {
    pub fn init(path: PathBuf, name: String) {
        let project = Self {
            path: path.clone(),
            config: Config { name },
        };

        let contents = toml::to_string(&project.config).expect("Failed to serialize Config.toml");

        let mut config_file =
            File::create(project.path.join("Config.toml")).expect("Failed to create Config.toml");

        config_file
            .write_all(contents.as_bytes())
            .expect("Failed to write to Config.toml");

        println!("Initialized project in {}", path.display());
    }

    pub fn load(path: PathBuf) -> Self {
        let mut config_file =
            File::open(path.join("Config.toml")).expect("Failed to open Config.toml file");

        let mut contents = String::new();
        config_file
            .read_to_string(&mut contents)
            .expect("Failed to read Config.toml");

        let config: Config = toml::from_str(&contents).expect("Failed to parse Config.toml file");
        Self { path, config }
    }

    pub fn save(&self) {
        let contents = toml::to_string(&self.config).expect("Failed to serialize Config.toml");

        let mut config_file =
            File::open(self.path.join("Config.toml")).expect("Failed to create Config.toml");

        config_file
            .write_all(contents.as_bytes())
            .expect("Failed to write to Config.toml");
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
}

// impl Serialize for Config {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(&self.name)
//     }
// }

// impl<'de> Deserialize<'de> for Config {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         Ok(Self {
//             name: deserializer.deserialize_str(visitor)
//         })
//     }
// }
