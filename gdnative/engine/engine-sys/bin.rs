use engine_sys::constants::Constants;
use engine_sys::generic::particle::*;

fn main() {
    println!("Beginning test");

    let descriptor = ParticleSolverDescriptor {
        constants: Constants {
            gravitational: 1.0,
            speed_of_light: 1.0,
        },
        particle_count: 0,
        particles: std::ptr::null(),
        element_order: 1,
        domain_width: 1.0,
        domain_height: 1.0,
        domain_depth: 1.0,
        domain_refinement: 2,
    };

    unsafe {
        let solver = particle_solver_create(descriptor);

        particle_solver_destroy(solver);
    }
}
