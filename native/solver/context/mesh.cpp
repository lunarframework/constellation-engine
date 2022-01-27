#include "base/defines.h"

#include "context/mesh.hpp"

#include <deal.II/grid/grid_generator.h>
#include <deal.II/grid/tria.h>

using namespace dealii;

SOLVER_API Mesh *mesh_create_uniform_grid(MeshUniformDescriptor descriptor)
{
    Mesh *p_mesh = new Mesh{};
    p_mesh->tria = Triangulation<3, 3>{};

    auto p1 = Point<3>(descriptor.centerx - descriptor.width / 2.0, descriptor.centery - descriptor.height / 2.0, descriptor.centerz - descriptor.depth / 2.0);
    auto p2 = Point<3>(descriptor.centerx + descriptor.width / 2.0, descriptor.centery + descriptor.height / 2.0, descriptor.centerz + descriptor.depth / 2.0);

    GridGenerator::hyper_rectangle(p_mesh->tria, p1, p2);
    p_mesh->tria.refine_global(descriptor.levels);

    return p_mesh;
}

SOLVER_API void mesh_destroy(Mesh *p_mesh)
{
    delete p_mesh;
}

SOLVER_API unsigned int mesh_n_active_cells(Mesh *p_mesh)
{
    return p_mesh->tria.n_active_cells();
}