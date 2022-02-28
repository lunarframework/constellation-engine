#include "mfem.hpp"

#include "export.hpp"
#include "constants.hpp"

#include <vector>

using namespace mfem;

struct Particle
{
    double x, y, z;
    double velx, vely, velz;
    double mass;
};

struct ParticleSolverDescriptor
{
    Constants constants;

    double domain_width;
    double domain_hieght;
    double domain_depth;
    int domain_refinement;
};

struct ParticleSolver
{
    Constants constants;
    std::vector<Particle> particles;

    Mesh *mesh;
};

ENGINE_API void *particle_solver_create(ParticleSolverDescriptor desc)
{
    ParticleSolver *solver = new ParticleSolver();
    solver->constants = desc.constants;

    ////////////////////////
    // Create Mesh /////////
    ////////////////////////

    solver->mesh = new Mesh(3, 8, 1);

    double hex_v[8][3] =
        {
            {-1, -1, -1}, {+1, -1, -1}, {+1, +1, -1}, {-1, +1, -1}, {-1, -1, +1}, {+1, -1, +1}, {+1, +1, +1}, {-1, +1, +1}};

    for (int i = 0; i < 8; i++)
    {
        hex_v[i][0] *= desc.domain_width;
        hex_v[i][1] *= desc.domain_hieght;
        hex_v[i][2] *= desc.domain_depth;

        solver->mesh->AddVertex(hex_v[i]);
    }

    int hex_e[8] =
        {0, 1, 2, 3, 4, 5, 6, 7};

    solver->mesh->AddHex(hex_e, 1);

    solver->mesh->FinalizeHexMesh(1, 1, true);

    for (int i = 0; i < desc.domain_refinement; i++)
    {
        solver->mesh->UniformRefinement();
    }

    return solver;
}

ENGINE_API unsigned int particle_solver_add_particle(void *p_solver, Particle particle)
{
    ParticleSolver *solver = (ParticleSolver *)p_solver;
    unsigned int index = solver->particles.size();
    solver->particles.push_back(particle);
    return index;
}

ENGINE_API Particle particle_solver_get_particle(void *p_solver, unsigned int index)
{
    ParticleSolver *solver = (ParticleSolver *)p_solver;

    return solver->particles[index];
}

ENGINE_API void particle_solver_setup(void *p_solver, double start, double end)
{
}

ENGINE_API void particle_solver_destroy(void *p_solver)
{
    delete (ParticleSolver *)p_solver;
}
