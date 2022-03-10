pub mod base;
pub mod global;
pub mod gravity;
pub mod scripts;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[test]
    fn test() {
        use super::*;
        use base::*;
        use glam::DVec3;
        use global::Units;
        use gravity::nbody::*;
        use gravity::*;
        use scripts::system_tree::*;
        use std::io::{Read, Seek, SeekFrom, Write};

        let path = std::env::current_dir().unwrap().join("example.cesystem");
        println!("Writing to {:?}", path);

        let mut tree = SystemTree::new(GravitationalSystem);
        tree.config_mut().insert(Units::default());

        let mut nbodies = SystemNode::new(NBodySystem);

        nbodies.children_mut().spawn((
            NBody {
                index: 0,
                pos: DVec3::new(5.0, 0.0, 0.0),
                vel: DVec3::new(0.0, 0.5, 0.0),
                mass: 0.5,
            },
            ContinuousRecord::<Position>::new(),
        ));

        nbodies.children_mut().spawn((
            NBody {
                index: 1,
                pos: DVec3::new(0.0, 0.0, 0.0),
                vel: DVec3::new(0.0, 0.0, 0.0),
                mass: 2.0,
            },
            ContinuousRecord::<Position>::new(),
        ));

        nbodies.children_mut().spawn((
            NBody {
                index: 2,
                pos: DVec3::new(0.0, 4.0, 0.0),
                vel: DVec3::new(-0.6, -0.5, 0.0),
                mass: 0.01,
            },
            ContinuousRecord::<Position>::new(),
        ));

        tree.root_mut().children_mut().spawn((nbodies,));

        tree.solve(0.0, 50.0, 10000);

        let system_tree =
            SystemTreeGD::new("Gravitational System".into(), SystemTreeRoot::Grav(tree));

        let contents = bincode::serialize(&system_tree).unwrap();

        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&contents).unwrap();
    }
}
