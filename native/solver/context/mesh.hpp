#pragma once

#include <deal.II/grid/tria.h>

#include "stdint.h"
#include "base/defines.h"

struct Mesh
{
    dealii::Triangulation<3, 3> tria;
};

typedef struct MeshUniformDescriptor
{
    double centerx, centery, centerz;

    double width;
    double height;
    double depth;

    unsigned int levels;

} MeshUniformDescriptor;

// Public interface

SOLVER_API Mesh *mesh_create_uniform(MeshUniformDescriptor descriptor);

SOLVER_API void mesh_destroy(Mesh *p_mesh);

SOLVER_API unsigned int mesh_n_active_cells(Mesh *p_mesh);
