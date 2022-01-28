use gdnative::api::ResourceLoader;
use gdnative::core_types::Vector3;
use gdnative::prelude::*;

use constellation_base::project::{Project, View};

use std::path::PathBuf;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct ProjectManager {
    project: Option<Project>,
    view: Option<View>,

    star_prefab: Option<Ref<PackedScene>>,
    stars: Vec<Ref<Node>>,
}

impl ProjectManager {
    /// The "constructor" of the class.
    fn new(_owner: &Spatial) -> Self {
        Self {
            project: None,
            view: None,
            star_prefab: None,

            stars: Vec::new(),
        }
    }
}

#[methods]
impl ProjectManager {
    #[export]
    fn _ready(&mut self, owner: TRef<Spatial>) {
        // // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
        godot_print!("Registering Signal");

        unsafe {
            owner
                .get_node("../GUI/OpenProject")
                .unwrap()
                .assume_safe()
                .connect(
                    "dir_selected",
                    owner,
                    "on_dir_selected",
                    VariantArray::new_shared(),
                    1,
                )
                .unwrap();

            owner
                .get_node("../GUI/BottomPanel/HSplitContainer/TimeSlider")
                .unwrap()
                .assume_safe()
                .connect(
                    "value_changed",
                    owner,
                    "on_time_set",
                    VariantArray::new_shared(),
                    1,
                )
                .unwrap();
        }

        self.star_prefab = Some(
            ResourceLoader::godot_singleton()
                .load("res://celestial/star.tscn", "PackedScene", false)
                .unwrap()
                .cast::<PackedScene>()
                .unwrap(),
        );
    }

    #[export]
    fn on_dir_selected(&mut self, owner: TRef<Spatial>, dir: GodotString) {
        godot_print!("Loading project from {}", dir);

        godot_print!("Clearing Children");

        self.stars.clear();

        for star in owner.get_children().iter() {
            let child = star.try_to_object::<Node>().unwrap();
            unsafe { child.assume_safe() }.queue_free();
        }

        let path = PathBuf::from(dir.to_string());

        match Project::load(path.clone()) {
            Ok(project) => {
                godot_print!("Found Project");
                self.project = Some(project);
                if self.project.as_ref().unwrap().views.is_empty() {
                    godot_print!("No views in project");
                    return;
                }
                self.view = Some(
                    self.project
                        .as_ref()
                        .unwrap()
                        .views
                        .first()
                        .unwrap()
                        .1
                        .clone(),
                );

                godot_print!("Creating children");

                for star in self.view.as_ref().unwrap().stars.iter() {
                    let reference = unsafe {
                        self.star_prefab
                            .as_ref()
                            .unwrap()
                            .assume_safe()
                            .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
                            .unwrap()
                    };

                    self.stars.push(reference);

                    let instance = unsafe { reference.assume_safe() };

                    instance
                        .cast::<Spatial>()
                        .unwrap()
                        .set_translation(Vector3::new(
                            star.pos.x as f32,
                            star.pos.y as f32,
                            star.pos.z as f32,
                        ));

                    owner.add_child(instance, false);
                }
            }
            Err(error) => {
                godot_print!(
                    "Failed to open project at {:?}, with error {:?}",
                    path,
                    error
                );
            }
        };
    }

    #[export]
    fn on_time_set(&mut self, _owner: &Spatial, value: f32) {
        if self.view.is_some() {
            let time = value as f64 * self.view.as_ref().unwrap().max_time;

            godot_print!("Time: {}", time);

            let time_data_n = self.view.as_ref().unwrap().steps + 1;

            let fractional_index = time_data_n as f32 * value;

            let old_index = (time_data_n as f32 * value).floor() as u32;
            let new_index = (time_data_n as f32 * value).ceil() as u32;

            let interp = (fractional_index - old_index as f32) / (new_index - old_index) as f32;

            for i in 0..self.stars.len() {
                let old_data =
                    &self.view.as_ref().unwrap().data[old_index as usize * self.stars.len() + i];

                let new_data =
                    &self.view.as_ref().unwrap().data[new_index as usize * self.stars.len() + i];

                let x = old_data.pos.x as f32 * interp + new_data.pos.x as f32 * (1.0 - interp);
                let y = old_data.pos.y as f32 * interp + new_data.pos.y as f32 * (1.0 - interp);
                let z = old_data.pos.z as f32 * interp + new_data.pos.z as f32 * (1.0 - interp);

                godot_print!("{:?}", (x, y, z));

                unsafe { self.stars[i].assume_safe() }
                    .cast::<Spatial>()
                    .unwrap()
                    .set_translation(Vector3::new(x, y, z));
            }
        }
    }
}
