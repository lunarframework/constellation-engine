use crate::base::SystemTree;
use crate::gravity::GravitationalSystem;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SystemHierarchyRoot {
    Grav(SystemTree<GravitationalSystem>),
}

#[derive(Serialize, Deserialize)]
pub struct SystemHierarchyFormat {
    pub name: String,
    pub root: SystemHierarchyRoot,
}

#[derive(NativeClass)]
#[inherit(Reference)]
#[no_constructor]
pub struct SystemHierarchy(Option<SystemHierarchyFormat>);

impl SystemHierarchy {
    pub fn new(inner: SystemHierarchyFormat) -> Self {
        Self(Some(inner))
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn inner(&self) -> Option<&SystemHierarchyFormat> {
        self.0.as_ref()
    }

    pub fn inner_mut(&mut self) -> Option<&mut SystemHierarchyFormat> {
        self.0.as_mut()
    }
}

#[methods]
impl SystemHierarchy {
    #[export]
    fn is_empty(&self, _onwer: &Reference) -> bool {
        self.0.is_none()
    }

    #[export]
    fn name(&self, _onwer: &Reference) -> Option<GodotString> {
        self.inner()
            .map(|format| GodotString::from_str(&format.name))
    }
}
