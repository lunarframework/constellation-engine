use gdnative::api::ResourceLoader;
use gdnative::core_types::Vector3;
use gdnative::prelude::*;

use constellation_base::project::{Project, View};

use std::path::PathBuf;

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct ProjectManager {
    project: Option<Project>,

    star_prefab: Option<Ref<PackedScene>>,
    stars: Vec<Ref<Node>>,
}

#[methods]
impl ProjectManager {
    /// The "constructor" of the class.
    fn new(owner: &Reference) -> Self {
        Self {
            project: None,

            star_prefab: None,
            stars: Vec::new(),
        }
    }

    #[export]
    fn open(&mut self, _owner: &Reference, dir: GodotString) -> GodotString {
        godot_print!("Loading constellation project from {}", dir);

        match Project::load(PathBuf::from(dir.to_string())) {
            Ok(project) => {
                self.project = Some(project);
                GodotString::from_str("")
            }
            Err(error) => GodotString::from_str(&format!("{}", error)),
        }
    }

    #[export]
    fn is_open(&self, _owner: &Reference) -> bool {
        self.project.is_some()
    }

    #[export]
    fn save(&mut self, _owner: &Reference) {
        if let Some(project) = self.project.as_ref() {}
    }

    #[export]
    fn close(&mut self, _owner: &Reference) {
        self.project = None;
    }
}
