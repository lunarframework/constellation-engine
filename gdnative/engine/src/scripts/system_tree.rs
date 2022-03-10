use crate::base::ContinuousRecord;
use crate::base::{SystemNode, SystemTree};
use crate::gravity::nbody::Position;
use crate::gravity::nbody::{NBody, NBodySystem};
use crate::gravity::GravitationalSystem;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SystemTreeRoot {
    None,
    Grav(SystemTree<GravitationalSystem>),
}

#[derive(NativeClass)]
#[inherit(Reference)]
#[no_constructor]
#[derive(Serialize, Deserialize)]
pub struct SystemTreeGD {
    pub name: String,
    pub root: SystemTreeRoot,
}

impl SystemTreeGD {
    pub fn new(name: String, root: SystemTreeRoot) -> Self {
        Self { name, root }
    }

    pub fn empty() -> Self {
        Self {
            name: String::from("Empty"),
            root: SystemTreeRoot::None,
        }
    }
}

#[methods]
impl SystemTreeGD {
    #[export]
    fn is_none(&self, _onwer: &Reference) -> bool {
        if let SystemTreeRoot::None = self.root {
            return true;
        }
        false
    }

    #[export]
    fn name(&self, _onwer: &Reference) -> GodotString {
        GodotString::from_str(&self.name)
    }

    #[export]
    fn positions(&mut self, _owner: &Reference, time: f64) -> VariantArray<Unique> {
        let mut vector = Vec::new();

        if let SystemTreeRoot::Grav(ref mut tree) = self.root {
            if let Some((_e, nbody)) = tree
                .root_mut()
                .children_mut()
                .query_mut::<&mut SystemNode<NBodySystem>>()
                .into_iter()
                .next()
            {
                for (_e, (body, record)) in nbody
                    .children_mut()
                    .query_mut::<(&NBody, &ContinuousRecord<Position>)>()
                {
                    let pos = record.load(time);
                    vector.push((body.index, pos.pos));
                }
            }
        }

        vector.sort_by(|a, b| a.0.cmp(&b.0));

        let array = VariantArray::new();

        for v in vector {
            array.push(Vector3::new(v.1.x as f32, v.1.y as f32, v.1.z as f32));
        }

        array
    }
}
