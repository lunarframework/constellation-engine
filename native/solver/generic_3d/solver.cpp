#include <cstdlib>
#include <cstdint>

#include <deal.II/grid/tria.h>
#include <deal.II/grid/grid_generator.h>

#include "base/grids.h"
#include "generic_3d/solver_types.h"

#define CUBE_GRID 0

struct Solver3d_T
{
    // Settings
    union
    {
        CubeGrid cube;
    } grid;
    int grid_type;

    // Output

    uint32_t n_vertices;
};

extern "C" Solver3d create_solver_3d()
{
    Solver3d_T *p_solver = new Solver3d_T{};

    p_solver->grid_type = CUBE_GRID;
    p_solver->grid.cube.width = 1.0;
    p_solver->grid.cube.height = 1.0;
    p_solver->grid.cube.depth = 1.0;
    p_solver->grid.cube.refinement = 0;

    return p_solver;
}

extern "C" void set_solver_3d_cube_grid(Solver3d solver, CubeGrid cube)
{
    solver->grid_type = CUBE_GRID;
    solver->grid.cube = cube;
}

extern "C" void run_solver_3d(Solver3d solver)
{
    using namespace dealii;

    Triangulation<3> traingulation;

    if (solver->grid_type == CUBE_GRID)
    {
        double width = solver->grid.cube.width;
        double height = solver->grid.cube.height;
        double depth = solver->grid.cube.depth;

        GridGenerator::hyper_rectangle(traingulation, Point<3>{-width / 2.0, -height / 2.0, -depth / 2.0}, Point<3>{width / 2.0, height / 2.0, depth / 2.0});
        traingulation.refine_global((unsigned int)solver->grid.cube.refinement);
    }
}

extern "C" void destroy_solver_3d(Solver3d solver)
{
    delete (Solver3d_T *)solver;
}
