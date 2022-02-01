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

    #[export]
    fn spawn_initial_data(
        &mut self,
        _owner: &Reference,
        star_prefab: Ref<PackedScene>,
        dst: Ref<Node>,
    ) {
        if let Some(project) = self.project.as_ref() {
            let dst = unsafe { dst.assume_safe() };
            for star in project.stars() {
                let spatial = unsafe {
                    star_prefab
                        .assume_safe()
                        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
                        .unwrap()
                        .assume_safe()
                        .cast::<Spatial>()
                        .unwrap()
                };

                spatial.set_translation(Vector3::new(
                    star.pos.x as f32,
                    star.pos.y as f32,
                    star.pos.z as f32,
                ));

                dst.add_child(spatial, false);
            }
        }
    }

    #[export]
    fn views(&mut self, _owner: &Reference) -> VariantArray<Shared> {
        let array = VariantArray::<Unique>::new();

        if let Some(project) = self.project.as_ref() {
            for (name, _view) in project.views() {
                array.push(GodotString::from_str(name));
            }
        }

        array.into_shared()
    }
}
