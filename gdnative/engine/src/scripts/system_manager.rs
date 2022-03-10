use super::{GravDescriptor, SystemTreeGD, SystemTreeRoot};
use super::{SolveDescriptor, UnitsDescriptor};
use crate::base::SystemTree;
use crate::gravity::GravitationalSystem;
use gdnative::api::Tree;
use gdnative::prelude::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to open file")]
    FileSystemError,
    #[error("Failed to deserialize contents of file")]
    DeserializationError,
}

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("FileSystem Error")]
    FileSystemError,
    #[error("Failed to serialize tree")]
    SerializationError,
}

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct SystemManager {}

#[methods]
impl SystemManager {
    /// The "constructor" of the class.
    fn new(_owner: &Reference) -> Self {
        Self {}
    }

    // #[export]
    // fn create_grav(
    //     &self,
    //     _owner: &Reference,
    //     desc: Instance<GravDescriptor, Shared>,
    // ) -> Instance<SystemTreeGD, Unique> {
    //     let desc = unsafe { desc.assume_safe() };

    //     let desc =
    //         match desc.map(|desc: &GravDescriptor, _base: TRef<Reference, Shared>| desc.clone()) {
    //             Err(error) => {
    //                 godot_error!("Failed to access descriptor with error {:?}", error);
    //                 GravDescriptor::default()
    //             }
    //             Ok(desc) => desc,
    //         };

    //     SystemTreeGD::new(
    //         desc.name,
    //         SystemTreeRoot::Grav(SystemTree::new(GravitationalSystem)),
    //     )
    //     .emplace()
    // }

    #[export]
    fn load(&self, _owner: &Reference, path: GodotString) -> Instance<SystemTreeGD, Unique> {
        let path = PathBuf::from(path.to_string());

        match load_hierarchy(path.clone()) {
            Ok(tree) => tree.emplace(),
            Err(error) => {
                godot_error!("Failed to load hierarchy with error {:?}", error);
                SystemTreeGD::empty().emplace()
            }
        }
    }

    #[export]
    fn save(
        &self,
        _owner: &Reference,
        _path: GodotString,
        _system_tree: Instance<SystemTreeGD, Shared>,
    ) {
        // let path = PathBuf::from(path.to_string());
        // let system_tree = unsafe { system_tree.assume_safe() };

        // match system_tree.map(|hierarchy: &SystemTreeGD, _base: TRef<Reference, Shared>| {
        //     match save_hierarchy(path, hierarchy) {
        //         Err(error) => {
        //             godot_error!("Failed to save hierarchy with error {:?}", error);
        //         }
        //         _ => {}
        //     };
        // }) {
        //     Err(error) => {
        //         godot_error!("Failed to access hierarchy with error {:?}", error);
        //     }
        //     _ => {}
        // }
    }
}

fn load_hierarchy(path: PathBuf) -> Result<SystemTreeGD, LoadError> {
    let mut file = File::open(&path).map_err(|_error| LoadError::FileSystemError)?;

    let len = file
        .seek(SeekFrom::End(0))
        .map_err(|_error| LoadError::FileSystemError)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_error| LoadError::FileSystemError)?;

    let mut contents = vec![0u8; len as usize];

    file.read(&mut contents)
        .map_err(|_error| LoadError::FileSystemError)?;

    Ok(bincode::deserialize(&contents).map_err(|_e| LoadError::DeserializationError)?)
}

fn save_hierarchy(path: PathBuf, tree: &SystemTreeGD) -> Result<(), SaveError> {
    let contents = bincode::serialize(tree).map_err(|_e| SaveError::SerializationError)?;

    let mut file = File::create(path).map_err(|_e| SaveError::FileSystemError)?;
    file.write_all(&contents)
        .map_err(|_e| SaveError::FileSystemError)?;

    Ok(())
}
