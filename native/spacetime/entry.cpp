#include <deal.II/grid/tria.h>
#include <deal.II/grid/grid_generator.h>
#include <iostream>
#include <fstream>
#include <cmath>

using namespace dealii;

void first_grid()
{
    Triangulation<2> triangulation;
    GridGenerator::hyper_cube(triangulation);
    triangulation.refine_global(4);

    std::cout << triangulation.n_vertices() << std::endl;
}
