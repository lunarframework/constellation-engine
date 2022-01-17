use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod config;
mod data;
mod view;

pub use config::{Config, Length, Mass, Time};
pub use data::Data;
pub use view::View;

pub struct Project {
    pub path: PathBuf,
    pub config: Config,
    pub data: Data,
    pub views: Vec<(String, View)>,
}

impl Project {
    pub fn init(path: PathBuf, name: String) -> std::io::Result<()> {
        let project = Self {
            path: path.clone(),
            config: Config {
                name,
                ..Default::default()
            },
            data: Data {},
            views: Vec::new(),
        };

        {
            let contents =
                toml::to_string(&project.config).expect("Failed to serialize Config.toml");

            let mut file = File::create(project.path.join("Config.toml"))?;

            file.write_all(contents.as_bytes())?
        }

        {
            let contents = toml::to_string(&project.data).expect("Failed to serialize data.bin");
            let mut file = File::create(project.path.join("Data.toml"))?;

            file.write_all(contents.as_bytes())?;
        }

        std::fs::create_dir(project.path.join("views"))?;

        println!("Initialized project in {}", path.display());

        Ok(())
    }

    pub fn load(path: PathBuf) -> std::io::Result<Self> {
        let config = {
            let mut file = File::open(path.join("Config.toml"))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            toml::from_str(&contents).expect("Failed to parse Config.toml file")
        };

        let data = {
            let mut file = File::open(path.join("Data.toml"))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            toml::from_str(&contents)?
        };

        let views = {
            let path = path.join("views");
            if path.exists() {
                let mut views = Vec::new();
                for i in std::fs::read_dir(&path).unwrap() {
                    if let Ok(entry) = i {
                        if entry.path().is_file() {
                            let mut file = File::open(entry.path())?;

                            let mut contents = String::new();
                            file.read_to_string(&mut contents)?;

                            views.push((
                                entry.file_name().into_string().unwrap(),
                                toml::from_str(&contents).expect("Failed to parse view file"),
                            ));
                        }
                    }
                }

                views
            } else {
                Vec::new()
            }
        };

        Ok(Self {
            path,
            config,
            data,
            views,
        })
    }

    pub fn save(&self) -> std::io::Result<()> {
        {
            let contents = toml::to_string(&self.config).expect("Failed to serialize Config.toml");

            let mut file = File::create(self.path.join("Config.toml"))?;

            file.write_all(contents.as_bytes())?;
        }

        {
            let contents = toml::to_string(&self.data).expect("Failed to serialize Data.toml");
            let mut file = File::create(self.path.join("data.bin"))?;

            file.write_all(contents.as_bytes())?;
        }

        {
            let path = self.path.join("views");

            if !path.exists() {
                std::fs::create_dir(&path)?;
            }

            for (name, view) in self.views.iter() {
                let contents = bincode::serialize(view).expect("Failed to serialize view file");

                let mut file = File::create(self.path.join(format!("{}.bin", name)))?;

                file.write_all(&contents)?;
            }
        }

        Ok(())
    }
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
