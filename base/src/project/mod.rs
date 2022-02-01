use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

mod config;
mod data;
pub mod view;

pub use config::{Length, Mass, Time, Units};
pub use data::Data;
pub use view::View;

use crate::Star;
use config::Config;

pub struct Project {
    path: PathBuf,
    config: Config,
    pub data: Data,
    pub views: HashMap<String, View>,
}

impl Project {
    pub fn init(path: PathBuf, name: String) -> Result<(), Box<dyn Error>> {
        let project = Self {
            path: path.clone(),
            config: Config {
                name,
                ..Default::default()
            },
            data: Data::default(),
            views: HashMap::new(),
        };

        {
            let contents = toml::to_string(&project.config)?;

            let mut file = File::create(project.path.join("Config.toml"))?;

            file.write_all(contents.as_bytes())?
        }

        {
            let contents = toml::to_string(&project.data)?;
            let mut file = File::create(project.path.join("Data.toml"))?;

            file.write_all(contents.as_bytes())?;
        }

        std::fs::create_dir(project.path.join("views"))?;

        println!("Initialized project in {}", path.display());

        Ok(())
    }

    pub fn load(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        println!("");
        let config = {
            let mut file = File::open(path.join("Config.toml"))?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            toml::from_str(&contents)?
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
                let mut views = HashMap::new();
                for i in std::fs::read_dir(&path).unwrap() {
                    if let Ok(entry) = i {
                        if entry.path().is_file() {
                            if let Ok(name) = entry.file_name().into_string() {
                                let view = bincode::deserialize(&std::fs::read(entry.path())?)?;
                                views.insert(name, view);
                            }
                        }
                    }
                }

                views
            } else {
                HashMap::new()
            }
        };

        Ok(Self {
            path,
            config,
            data,
            views,
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        {
            let contents = toml::to_string(&self.config)?;

            let mut file = File::create(self.path.join("Config.toml"))?;

            file.write_all(contents.as_bytes())?;
        }

        {
            let contents = toml::to_string(&self.data)?;
            let mut file = File::create(self.path.join("Data.toml"))?;

            file.write_all(contents.as_bytes())?;
        }

        {
            let path = self.path.join("views");

            if !path.exists() {
                std::fs::create_dir(&path)?;
            }

            for (name, view) in self.views.iter() {
                let contents = bincode::serialize(view)?;

                let mut file = File::create(path.join(format!("{}.bin", name)))?;

                file.write_all(&contents)?;
            }
        }

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.config.name = String::from(name);
    }

    pub fn units(&self) -> &Units {
        &self.config.units
    }

    pub fn set_units(&mut self, units: Units) {
        self.config.units = units;
    }

    pub fn insert_view(&mut self, name: String, view: View) {
        self.views.insert(name, view);
    }

    pub fn get_view(&self, name: &String) -> Option<&View> {
        self.views.get(name)
    }

    pub fn get_view_mut(&mut self, name: &String) -> Option<&mut View> {
        self.views.get_mut(name)
    }

    pub fn views(&self) -> std::collections::hash_map::Iter<'_, String, View> {
        self.views.iter()
    }

    pub fn views_mut(&mut self) -> std::collections::hash_map::IterMut<'_, String, View> {
        self.views.iter_mut()
    }

    pub fn add_star(&mut self, star: Star) {
        self.data.stars.push(star);
    }

    pub fn stars(&self) -> std::slice::Iter<'_, Star> {
        self.data.stars.iter()
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
