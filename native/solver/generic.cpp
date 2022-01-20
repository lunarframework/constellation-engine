#include <cstdlib>
#include <cstdint>

#include <vector>

#include <deal.II/base/quadrature_lib.h>
#include <deal.II/base/tensor.h>
#include <deal.II/grid/tria.h>
#include <deal.II/grid/grid_generator.h>
#include <deal.II/grid/grid_tools.h>
#include <deal.II/dofs/dof_handler.h>
#include <deal.II/dofs/dof_tools.h>
#include <deal.II/dofs/dof_renumbering.h>
#include <deal.II/fe/fe_q.h>
#include <deal.II/fe/fe_values.h>
#include <deal.II/lac/affine_constraints.h>
#include <deal.II/lac/vector.h>
#include <deal.II/lac/full_matrix.h>
#include <deal.II/lac/sparse_matrix.h>
#include <deal.II/lac/dynamic_sparsity_pattern.h>
#include <deal.II/lac/solver_cg.h>
#include <deal.II/lac/precondition.h>

#include "base/grids.h"
#include "base/defines.h"
#include "base/nbody.h"

typedef struct GenericSolver_T *GenericSolver;

#define CUBE_GRID 0

struct GenericSolver_T
{
    // Settings
    union
    {
        CubeGrid cube;
    } grid;
    int grid_type;

    // Objevts

    std::vector<NBody> nbodies;

    // Output

    uint32_t n_vertices;
};

SOLVER_API GenericSolver create_generic_solver()
{
    GenericSolver_T *p_solver = new GenericSolver_T{};

    p_solver->grid_type = CUBE_GRID;
    p_solver->grid.cube.width = 1.0;
    p_solver->grid.cube.height = 1.0;
    p_solver->grid.cube.depth = 1.0;
    p_solver->grid.cube.refinement = 0;

    p_solver->nbodies = std::vector<NBody>();

    return p_solver;
}

SOLVER_API void generic_solver_add_nbody(GenericSolver solver, NBody nbody)
{
    solver->nbodies.push_back(nbody);
}

SOLVER_API void run_generic_solver(GenericSolver solver)
{
    using namespace dealii;

    // Create Grid/Domain

    Triangulation<3> triangulation;

    GridGenerator::hyper_rectangle(triangulation, Point<3>{-1.0, -1.0, -1.0}, Point<3>{1.0, 1.0, 1.0});
    triangulation.refine_global(5);

    GridTools::Cache<3, 3> cache{triangulation};

    // Assign dofs
    DoFHandler<3> dof_handler{triangulation};

    const FE_Q<3> fe{1};
    dof_handler.distribute_dofs(fe);

    // Renumber dofs

    DoFRenumbering::Cuthill_McKee(dof_handler);

    QGauss<3> quadrature_formula{fe.degree + 1};

    FEValues<3> fe_values{fe,
                          quadrature_formula,
                          update_values | update_gradients | update_JxW_values};

    // Setup

    const auto n_dofs = dof_handler.n_dofs();
    const auto n_dofs_per_cell = fe.n_dofs_per_cell();

    double delta_time = 1.0;
    uint32_t max_iteration = 0;

    // Spacetime

    auto metric11 = Vector<double>{n_dofs};
    auto metric12 = Vector<double>{n_dofs};
    auto metric13 = Vector<double>{n_dofs};
    auto metric22 = Vector<double>{n_dofs};
    auto metric23 = Vector<double>{n_dofs};
    auto metric33 = Vector<double>{n_dofs};

    auto kext11 = Vector<double>{n_dofs};
    auto kext12 = Vector<double>{n_dofs};
    auto kext13 = Vector<double>{n_dofs};
    auto kext22 = Vector<double>{n_dofs};
    auto kext23 = Vector<double>{n_dofs};
    auto kext33 = Vector<double>{n_dofs};

    auto lapse = Vector<double>{n_dofs};

    // Energy-Momentum

    // Linear Algebra containers

    SparsityPattern sparsity_pattern{};

    {
        DynamicSparsityPattern dsp{n_dofs};
        DoFTools::make_sparsity_pattern(dof_handler, dsp);
        sparsity_pattern.copy_from(dsp);
    }

    SparseMatrix<double> shape_matrix{sparsity_pattern};

    Vector<double> metric11_rhs{n_dofs};
    Vector<double> metric12_rhs{n_dofs};
    Vector<double> metric13_rhs{n_dofs};
    Vector<double> metric22_rhs{n_dofs};
    Vector<double> metric23_rhs{n_dofs};
    Vector<double> metric33_rhs{n_dofs};

    Vector<double> kext11_rhs{n_dofs};
    Vector<double> kext12_rhs{n_dofs};
    Vector<double> kext13_rhs{n_dofs};
    Vector<double> kext22_rhs{n_dofs};
    Vector<double> kext23_rhs{n_dofs};
    Vector<double> kext33_rhs{n_dofs};

    Vector<double> metric11_rhs_sum{n_dofs};
    Vector<double> metric12_rhs_sum{n_dofs};
    Vector<double> metric13_rhs_sum{n_dofs};
    Vector<double> metric22_rhs_sum{n_dofs};
    Vector<double> metric23_rhs_sum{n_dofs};
    Vector<double> metric33_rhs_sum{n_dofs};

    Vector<double> kext11_rhs_sum{n_dofs};
    Vector<double> kext12_rhs_sum{n_dofs};
    Vector<double> kext13_rhs_sum{n_dofs};
    Vector<double> kext22_rhs_sum{n_dofs};
    Vector<double> kext23_rhs_sum{n_dofs};
    Vector<double> kext33_rhs_sum{n_dofs};

    // Scratch buffers

    auto metric11_values = std::vector<double>(n_dofs);
    auto metric12_values = std::vector<double>(n_dofs);
    auto metric13_values = std::vector<double>(n_dofs);
    auto metric22_values = std::vector<double>(n_dofs);
    auto metric23_values = std::vector<double>(n_dofs);
    auto metric33_values = std::vector<double>(n_dofs);

    auto metric11_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);
    auto metric12_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);
    auto metric13_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);
    auto metric22_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);
    auto metric23_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);
    auto metric33_gradients = std::vector<Tensor<1, 3, double>>(n_dofs);

    auto metric11_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);
    auto metric12_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);
    auto metric13_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);
    auto metric22_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);
    auto metric23_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);
    auto metric33_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);

    auto kext11_values = std::vector<double>(n_dofs);
    auto kext12_values = std::vector<double>(n_dofs);
    auto kext13_values = std::vector<double>(n_dofs);
    auto kext22_values = std::vector<double>(n_dofs);
    auto kext23_values = std::vector<double>(n_dofs);
    auto kext33_values = std::vector<double>(n_dofs);

    auto lapse_values = std::vector<double>(n_dofs);
    auto lapse_hessians = std::vector<Tensor<2, 3, double>>(n_dofs);

    auto energy_density = std::vector<double>(n_dofs);

    auto momentum_density1 = std::vector<double>(n_dofs);
    auto momentum_density2 = std::vector<double>(n_dofs);
    auto momentum_density3 = std::vector<double>(n_dofs);

    auto momentum_flux11_values = std::vector<double>(n_dofs);
    auto momentum_flux12_values = std::vector<double>(n_dofs);
    auto momentum_flux13_values = std::vector<double>(n_dofs);
    auto momentum_flux22_values = std::vector<double>(n_dofs);
    auto momentum_flux23_values = std::vector<double>(n_dofs);
    auto momentum_flux33_values = std::vector<double>(n_dofs);

    FullMatrix<double> cell_shape_matrix(n_dofs_per_cell, n_dofs_per_cell);

    Vector<double> cell_m11_rhs(n_dofs_per_cell);
    Vector<double> cell_m12_rhs(n_dofs_per_cell);
    Vector<double> cell_m13_rhs(n_dofs_per_cell);
    Vector<double> cell_m22_rhs(n_dofs_per_cell);
    Vector<double> cell_m23_rhs(n_dofs_per_cell);
    Vector<double> cell_m33_rhs(n_dofs_per_cell);

    Vector<double> cell_k11_rhs(n_dofs_per_cell);
    Vector<double> cell_k12_rhs(n_dofs_per_cell);
    Vector<double> cell_k13_rhs(n_dofs_per_cell);
    Vector<double> cell_k22_rhs(n_dofs_per_cell);
    Vector<double> cell_k23_rhs(n_dofs_per_cell);
    Vector<double> cell_k33_rhs(n_dofs_per_cell);

    std::vector<types::global_dof_index> local_dof_indices(n_dofs_per_cell);

    auto nbody_points = std::vector<Point<3, double>>();

    // Time loop
    for (uint32_t i = 0; i < max_iteration; i++)
    {

        // Update Source Terms

        {
            nbody_points.clear();
            nbody_points.reserve(solver->nbodies.size());

            for (auto &nbody : solver->nbodies)
            {
                nbody_points.push_back(Point<3, double>(nbody.x, nbody.y, nbody.z));
            }

            auto [cells, qpoints, indices] = GridTools::compute_point_locations(cache, nbody_points);

            for (int cell_index = 0; cell_index < cells.size(); cell_index++)
            {
                for (int point_index = 0; point_index < qpoints[cell_index].size(); point_index++)
                {
                    double total_distance = 0.0;

                    for (int vertex_index = 0; vertex_index < cells[cell_index]->n_vertices(); vertex_index++)
                    {
                        double distance = qpoints[cell_index][point_index].distance(cells[cell_index]->vertex(vertex_index));
                        total_distance += distance;
                    }

                    for (int vertex_index = 0; vertex_index < cells[cell_index]->n_vertices(); vertex_index++)
                    {
                        double weight = qpoints[cell_index][point_index].distance(cells[cell_index]->vertex(vertex_index)) / total_distance;

                        // Add to various source terms using data from nbodies, and multiply by weight
                    }
                }
            }
        }

        // Update values
        fe_values.get_function_values(metric11, metric11_values);
        fe_values.get_function_values(metric12, metric12_values);
        fe_values.get_function_values(metric13, metric13_values);
        fe_values.get_function_values(metric22, metric22_values);
        fe_values.get_function_values(metric23, metric23_values);
        fe_values.get_function_values(metric33, metric33_values);

        fe_values.get_function_gradients(metric11, metric11_gradients);
        fe_values.get_function_gradients(metric12, metric12_gradients);
        fe_values.get_function_gradients(metric13, metric13_gradients);
        fe_values.get_function_gradients(metric22, metric22_gradients);
        fe_values.get_function_gradients(metric23, metric23_gradients);
        fe_values.get_function_gradients(metric33, metric33_gradients);

        fe_values.get_function_hessians(metric11, metric11_hessians);
        fe_values.get_function_hessians(metric12, metric12_hessians);
        fe_values.get_function_hessians(metric13, metric13_hessians);
        fe_values.get_function_hessians(metric22, metric22_hessians);
        fe_values.get_function_hessians(metric23, metric23_hessians);
        fe_values.get_function_hessians(metric33, metric33_hessians);

        fe_values.get_function_values(kext11, kext11_values);
        fe_values.get_function_values(kext12, kext12_values);
        fe_values.get_function_values(kext13, kext13_values);
        fe_values.get_function_values(kext22, kext22_values);
        fe_values.get_function_values(kext23, kext23_values);
        fe_values.get_function_values(kext33, kext33_values);

        fe_values.get_function_values(lapse, lapse_values);
        fe_values.get_function_hessians(lapse, lapse_hessians);

        // Some Defines

#define ORDER_INDICES_00 00
#define ORDER_INDICES_01 01
#define ORDER_INDICES_02 02
#define ORDER_INDICES_03 03
#define ORDER_INDICES_10 01
#define ORDER_INDICES_11 11
#define ORDER_INDICES_12 12
#define ORDER_INDICES_13 13
#define ORDER_INDICES_20 02
#define ORDER_INDICES_21 12
#define ORDER_INDICES_22 22
#define ORDER_INDICES_23 23
#define ORDER_INDICES_30 03
#define ORDER_INDICES_31 13
#define ORDER_INDICES_32 23
#define ORDER_INDICES_33 33

#define INDICES(a, b) ORDER_INDICES_##a##b

        // Loop through cells
        for (const auto &cell : dof_handler.active_cell_iterators())
        {
            // Reset state
            fe_values.reinit(cell);

            cell_shape_matrix = 0;

            cell_m11_rhs = 0;
            cell_m12_rhs = 0;
            cell_m13_rhs = 0;
            cell_m22_rhs = 0;
            cell_m23_rhs = 0;
            cell_m33_rhs = 0;

            cell_k11_rhs = 0;
            cell_k12_rhs = 0;
            cell_k13_rhs = 0;
            cell_k22_rhs = 0;
            cell_k23_rhs = 0;
            cell_k33_rhs = 0;

            // Loop through quadrature points

            for (const auto q_index : fe_values.quadrature_point_indices())
            {

                // Update Shape Matrix from this point
                for (const auto i : fe_values.dof_indices())
                    for (const auto j : fe_values.dof_indices())
                        cell_shape_matrix(i, j) +=
                            (fe_values.shape_value(i, q_index) * // phi_i(x_q)
                             fe_values.shape_value(j, q_index) * // phi_j(x_q)
                             fe_values.JxW(q_index));            // dx

                        // Update RHS of both metric and kext

                        // Some Defines

                        // Stupid workaround to use macro in another macro

#define METRIC_VALUE(a, b) CONCAT(CONCAT(metric, INDICES(a, b)), _values[q_index])
#define METRIC_GRADIENT(a, b, c) CONCAT(CONCAT(metric, INDICES(a, b)), _gradients[q_index][c - 1])
#define METRIC_HESSSIAN(a, b, c, d) CONCAT(CONCAT(metric, INDICES(a, b)), _hessians[q_index][TableIndices<2>(c - 1, d - 1)])

#define KEXT_VALUE2(indices) kext##indices##_values[q_index]
#define KEXT_VALUE1(indices) KEXT_VALUE2(indices)
#define KEXT_VALUE(a, b) KEXT_VALUE1(INDICES(a, b))

#define LAPSE lapse[q_index]
#define LAPSE_HESSIAN(a, b) lapse_hessians[q_index][TableIndices<2>(a - 1, b - 1)]

                // Compute the determinate of the matrix

                auto mdet = metric11_values[q_index] * (metric22_values[q_index] * metric33_values[q_index] - metric23_values[q_index] * metric23_values[q_index]) -
                            metric12_values[q_index] * (metric12_values[q_index] * metric33_values[q_index] - metric23_values[q_index] * metric13_values[q_index]) +
                            metric13_values[q_index] * (metric12_values[q_index] * metric23_values[q_index] - metric22_values[q_index] * metric13_values[q_index]);

                auto inv_mdet = 1.0 / mdet;

                // Compute the inverse matrix

                auto minv11 = (metric22_values[q_index] * metric33_values[q_index] - metric23_values[q_index] * metric23_values[q_index]) * inv_mdet;
                auto minv12 = (metric13_values[q_index] * metric23_values[q_index] - metric12_values[q_index] * metric33_values[q_index]) * inv_mdet;
                auto minv13 = (metric12_values[q_index] * metric23_values[q_index] - metric13_values[q_index] * metric22_values[q_index]) * inv_mdet;
                auto minv22 = (metric11_values[q_index] * metric33_values[q_index] - metric13_values[q_index] * metric13_values[q_index]) * inv_mdet;
                auto minv23 = (metric12_values[q_index] * metric13_values[q_index] - metric11_values[q_index] * metric23_values[q_index]) * inv_mdet;
                auto minv33 = (metric11_values[q_index] * metric22_values[q_index] - metric12_values[q_index] * metric12_values[q_index]) * inv_mdet;

#define MINV2(indices) minv##indices
#define MINV1(indices) MINV2(indices)
#define MINV(i, j) MINV1(INDICES(i, j))

                // Compute the connection coefficients

#define CONNECTION_TERM(i, j, k, l) 0.5 * MINV(i, l) * (METRIC_GRADIENT(l, j, k) + METRIC_GRADIENT(l, k, j) - METRIC_GRADIENT(j, k, l))
#define COMPUTE_CONNECTION(i, j, k) CONNECTION_TERM(i, j, k, 1) + CONNECTION_TERM(i, j, k, 2) + CONNECTION_TERM(i, j, k, 3)

                auto connection11_1 = COMPUTE_CONNECTION(1, 1, 1);
                auto connection11_2 = COMPUTE_CONNECTION(1, 1, 2);
                auto connection11_3 = COMPUTE_CONNECTION(1, 1, 3);

                auto connection12_1 = COMPUTE_CONNECTION(1, 2, 1);
                auto connection12_2 = COMPUTE_CONNECTION(1, 2, 2);
                auto connection12_3 = COMPUTE_CONNECTION(1, 2, 3);

                auto connection13_1 = COMPUTE_CONNECTION(1, 3, 1);
                auto connection13_2 = COMPUTE_CONNECTION(1, 3, 2);
                auto connection13_3 = COMPUTE_CONNECTION(1, 3, 3);

                auto connection22_1 = COMPUTE_CONNECTION(2, 2, 1);
                auto connection22_2 = COMPUTE_CONNECTION(2, 2, 2);
                auto connection22_3 = COMPUTE_CONNECTION(2, 2, 3);

                auto connection23_1 = COMPUTE_CONNECTION(2, 3, 1);
                auto connection23_2 = COMPUTE_CONNECTION(2, 3, 2);
                auto connection23_3 = COMPUTE_CONNECTION(2, 3, 3);

                auto connection33_1 = COMPUTE_CONNECTION(3, 3, 1);
                auto connection33_2 = COMPUTE_CONNECTION(3, 3, 2);
                auto connection33_3 = COMPUTE_CONNECTION(3, 3, 3);

#define CONNECTION2(indices, c) connection##indices##_##c
#define CONNECTION1(indices, c) CONNECTION2(indices, c)
#define CONNECTION(a, b, c) CONNECTION1(INDICES(a, b), c)

                // Compute the Ricci Tensor

// TODO Add i == k or j == l optimization
#define RICCI_2ND_DER_TERM(i, j, k, l) 0.5 * MINV(k, l) * (METRIC_HESSSIAN(k, j, i, l) + METRIC_HESSSIAN(i, l, k, j) - METRIC_HESSSIAN(k, l, i, j) - METRIC_HESSSIAN(i, j, k, l))
#define RICCI_CONNECTION_TERM(i, j, k, l) CONNECTION(i, j, k) * CONNECTION(k, l, l) - CONNECTION(i, l, k) - CONNECTION(j, k, l)
#define COMPUTE_RICCI(i, j) RICCI_2ND_DER_TERM(i, j, 1, 1) + 2.0 * RICCI_2ND_DER_TERM(i, j, 1, 2) + 2.0 * RICCI_2ND_DER_TERM(i, j, 1, 3) +  \
                                RICCI_2ND_DER_TERM(i, j, 2, 2) + 2.0 * RICCI_2ND_DER_TERM(i, j, 2, 3) +                                     \
                                RICCI_2ND_DER_TERM(i, j, 3, 3) +                                                                            \
                                RICCI_CONNECTION_TERM(i, j, 1, 1) + RICCI_CONNECTION_TERM(i, j, 1, 2) + RICCI_CONNECTION_TERM(i, j, 1, 3) + \
                                RICCI_CONNECTION_TERM(i, j, 2, 1) + RICCI_CONNECTION_TERM(i, j, 2, 2) + RICCI_CONNECTION_TERM(i, j, 2, 3) + \
                                RICCI_CONNECTION_TERM(i, j, 3, 1) + RICCI_CONNECTION_TERM(i, j, 3, 2) + RICCI_CONNECTION_TERM(i, j, 3, 3)

                auto ricci11 = COMPUTE_RICCI(1, 1);
                auto ricci12 = COMPUTE_RICCI(1, 2);
                auto ricci13 = COMPUTE_RICCI(1, 3);
                auto ricci22 = COMPUTE_RICCI(2, 2);
                auto ricci23 = COMPUTE_RICCI(2, 3);
                auto ricci33 = COMPUTE_RICCI(3, 3);

#define RICCI2(indices) ricci##indices
#define RICCI1(indices) RICCI2(indices)
#define RICCI(a, b) RICCI1(INDICES(a, b))

                auto ktrace = MINV(1, 1) * KEXT_VALUE(1, 1) + 2.0 * MINV(1, 2) * KEXT_VALUE(1, 2) + 2.0 * MINV(1, 3) * KEXT_VALUE(1, 3) +
                              MINV(2, 2) * KEXT_VALUE(2, 2) + 2.0 * MINV(2, 3) * KEXT_VALUE(2, 3) +
                              MINV(3, 3) * KEXT_VALUE(3, 3);

#define KTRACE ktrace

                // Compute Metric RHS's

#define METRIC_RHS(i, j) -2.0 * KEXT_VALUE(i, j) * LAPSE

                auto m11_rhs = METRIC_RHS(1, 1);
                auto m12_rhs = METRIC_RHS(1, 2);
                auto m13_rhs = METRIC_RHS(1, 3);
                auto m22_rhs = METRIC_RHS(2, 2);
                auto m23_rhs = METRIC_RHS(2, 3);
                auto m33_rhs = METRIC_RHS(3, 3);

                // Compute kext RHS's

#define ENERGY_MOMENTUM(a, b) 0.0

#define COMPUTE_MOMENTUM_FLUX(i, j) METRIC_VALUE(i, 1) * METRIC_VALUE(j, 1) * ENERGY_MOMENTUM(1, 1) +     \
                                        METRIC_VALUE(i, 1) * METRIC_VALUE(j, 2) * ENERGY_MOMENTUM(1, 2) + \
                                        METRIC_VALUE(i, 1) * METRIC_VALUE(j, 3) * ENERGY_MOMENTUM(1, 3) + \
                                        METRIC_VALUE(i, 2) * METRIC_VALUE(j, 1) * ENERGY_MOMENTUM(2, 1) + \
                                        METRIC_VALUE(i, 2) * METRIC_VALUE(j, 2) * ENERGY_MOMENTUM(2, 2) + \
                                        METRIC_VALUE(i, 2) * METRIC_VALUE(j, 3) * ENERGY_MOMENTUM(2, 3) + \
                                        METRIC_VALUE(i, 3) * METRIC_VALUE(j, 1) * ENERGY_MOMENTUM(3, 1) + \
                                        METRIC_VALUE(i, 3) * METRIC_VALUE(j, 2) * ENERGY_MOMENTUM(3, 2) + \
                                        METRIC_VALUE(i, 3) * METRIC_VALUE(j, 3) * ENERGY_MOMENTUM(3, 3)

                auto energy_density = LAPSE * LAPSE * ENERGY_MOMENTUM(0, 0);

                /// Momentum density
                /// #define MOMENTUM_DENSITY(i)

                auto momentum_flux11 = COMPUTE_MOMENTUM_FLUX(1, 1);
                auto momentum_flux12 = COMPUTE_MOMENTUM_FLUX(1, 2);
                auto momentum_flux13 = COMPUTE_MOMENTUM_FLUX(1, 3);
                auto momentum_flux22 = COMPUTE_MOMENTUM_FLUX(2, 2);
                auto momentum_flux23 = COMPUTE_MOMENTUM_FLUX(2, 3);
                auto momentum_flux33 = COMPUTE_MOMENTUM_FLUX(3, 3);

#define ENERGY_DENSITY energy_density

#define MOMENTUM_FLUX2(indices) momentum_flux##indices
#define MOMENTUM_FLUX1(indices) MOMENTUM_FLUX2(indices)
#define MOMENTUM_FLUX(a, b) MOMENTUM_FLUX1(INDICES(a, b))

#define COMPUTE_MOMENTUM_FLUX_TRACE MINV(1, 1) * MOMENTUM_FLUX(1, 1) + MINV(1, 2) * MOMENTUM_FLUX(1, 2) + MINV(1, 3) * MOMENTUM_FLUX(1, 3) +     \
                                        MINV(2, 1) * MOMENTUM_FLUX(2, 1) + MINV(1, 2) * MOMENTUM_FLUX(2, 2) + MINV(2, 3) * MOMENTUM_FLUX(2, 3) + \
                                        MINV(3, 1) * MOMENTUM_FLUX(3, 1) + MINV(3, 2) * MOMENTUM_FLUX(3, 2) + MINV(3, 3) * MOMENTUM_FLUX(3, 3)

                auto momentum_flux_trace = COMPUTE_MOMENTUM_FLUX_TRACE;

#define MOMENTUM_FLUX_TRACE momentum_flux_trace

#define KTRACE_BY_KEXT(i, j) KTRACE *KEXT_VALUE(i, j)
#define KEXT_DOT_KEXT_TERM(i, j, k, l) KEXT_VALUE(i, k) * MINV(k, l) * KEXT_VALUE(l, j)
#define KEXT_DOT_KEXT(i, j) KEXT_DOT_KEXT_TERM(i, j, 1, 1) + KEXT_DOT_KEXT_TERM(i, j, 1, 2) + KEXT_DOT_KEXT_TERM(i, j, 1, 3) +     \
                                KEXT_DOT_KEXT_TERM(i, j, 2, 1) + KEXT_DOT_KEXT_TERM(i, j, 2, 2) + KEXT_DOT_KEXT_TERM(i, j, 3, 3) + \
                                KEXT_DOT_KEXT_TERM(i, j, 3, 1) + KEXT_DOT_KEXT_TERM(i, j, 3, 2) + KEXT_DOT_KEXT_TERM(i, j, 3, 3)

#define KEXT_RHS(i, j) LAPSE *(RICCI(i, j) - 2.0 * KEXT_DOT_KEXT(i, j) + KTRACE_BY_KEXT(i, j)) - LAPSE_HESSIAN(i, j) - \
                           8.0 * numbers::PI *LAPSE *(MOMENTUM_FLUX(i, j) - 1.0 / 2.0 * METRIC_VALUE(i, j) * (MOMENTUM_FLUX_TRACE - ENERGY_DENSITY))

                auto k11_rhs = KEXT_RHS(1, 1);
                auto k12_rhs = KEXT_RHS(1, 2);
                auto k13_rhs = KEXT_RHS(1, 3);
                auto k22_rhs = KEXT_RHS(2, 2);
                auto k23_rhs = KEXT_RHS(2, 3);
                auto k33_rhs = KEXT_RHS(3, 3);

                // Update cell contributions

                for (const auto i : fe_values.dof_indices())
                {
                    auto integrator = fe_values.shape_value(i, q_index) * fe_values.JxW(q_index);

                    cell_m11_rhs(i) += m11_rhs * integrator;
                    cell_m12_rhs(i) += m12_rhs * integrator;
                    cell_m13_rhs(i) += m13_rhs * integrator;
                    cell_m22_rhs(i) += m22_rhs * integrator;
                    cell_m23_rhs(i) += m23_rhs * integrator;
                    cell_m33_rhs(i) += m33_rhs * integrator;

                    cell_k11_rhs(i) += k11_rhs * integrator;
                    cell_k12_rhs(i) += k12_rhs * integrator;
                    cell_k13_rhs(i) += k13_rhs * integrator;
                    cell_k22_rhs(i) += k22_rhs * integrator;
                    cell_k23_rhs(i) += k23_rhs * integrator;
                    cell_k33_rhs(i) += k33_rhs * integrator;
                }
            }

            // Update Global matrices

            cell->get_dof_indices(local_dof_indices);
            for (const auto i : fe_values.dof_indices())
                for (const auto j : fe_values.dof_indices())
                    shape_matrix.add(local_dof_indices[i],
                                     local_dof_indices[j],
                                     cell_shape_matrix(i, j));

            for (const auto i : fe_values.dof_indices())
            {
                metric11_rhs(local_dof_indices[i]) += cell_m11_rhs(i);
                metric12_rhs(local_dof_indices[i]) += cell_m12_rhs(i);
                metric13_rhs(local_dof_indices[i]) += cell_m13_rhs(i);
                metric22_rhs(local_dof_indices[i]) += cell_m22_rhs(i);
                metric23_rhs(local_dof_indices[i]) += cell_m23_rhs(i);
                metric33_rhs(local_dof_indices[i]) += cell_m33_rhs(i);

                kext11_rhs(local_dof_indices[i]) += cell_k11_rhs(i);
                kext12_rhs(local_dof_indices[i]) += cell_k12_rhs(i);
                kext13_rhs(local_dof_indices[i]) += cell_k13_rhs(i);
                kext22_rhs(local_dof_indices[i]) += cell_k22_rhs(i);
                kext23_rhs(local_dof_indices[i]) += cell_k23_rhs(i);
                kext33_rhs(local_dof_indices[i]) += cell_k33_rhs(i);
            }
        }

        SolverControl solver_control(100, 1e-12);
        SolverCG<Vector<double>> solver(solver_control);

        // Update Metric

        shape_matrix.vmult(metric11_rhs_sum, metric11);
        metric11_rhs_sum.sadd(delta_time, metric11_rhs);
        shape_matrix.vmult(metric12_rhs_sum, metric12);
        metric12_rhs_sum.sadd(delta_time, metric12_rhs);
        shape_matrix.vmult(metric13_rhs_sum, metric13);
        metric13_rhs_sum.sadd(delta_time, metric13_rhs);
        shape_matrix.vmult(metric22_rhs_sum, metric22);
        metric22_rhs_sum.sadd(delta_time, metric22_rhs);
        shape_matrix.vmult(metric23_rhs_sum, metric23);
        metric23_rhs_sum.sadd(delta_time, metric23_rhs);
        shape_matrix.vmult(metric33_rhs_sum, metric33);
        metric33_rhs_sum.sadd(delta_time, metric33_rhs);

        solver.solve(shape_matrix, metric11, metric11_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, metric12, metric12_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, metric13, metric13_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, metric22, metric22_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, metric23, metric23_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, metric33, metric33_rhs_sum, PreconditionIdentity());

        // Update extrinsic curvature

        shape_matrix.vmult(kext11_rhs_sum, kext11);
        kext11_rhs_sum.sadd(delta_time, kext11_rhs);
        shape_matrix.vmult(kext12_rhs_sum, kext12);
        kext12_rhs_sum.sadd(delta_time, kext12_rhs);
        shape_matrix.vmult(kext13_rhs_sum, kext13);
        kext13_rhs_sum.sadd(delta_time, kext13_rhs);
        shape_matrix.vmult(kext22_rhs_sum, kext22);
        kext22_rhs_sum.sadd(delta_time, kext22_rhs);
        shape_matrix.vmult(kext23_rhs_sum, kext23);
        kext23_rhs_sum.sadd(delta_time, kext23_rhs);
        shape_matrix.vmult(kext33_rhs_sum, kext33);
        kext33_rhs_sum.sadd(delta_time, kext33_rhs);

        solver.solve(shape_matrix, kext11, kext11_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, kext12, kext12_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, kext13, kext13_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, kext22, kext22_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, kext23, kext23_rhs_sum, PreconditionIdentity());
        solver.solve(shape_matrix, kext33, kext33_rhs_sum, PreconditionIdentity());
    }
}

SOLVER_API void destroy_generic_solver(GenericSolver solver)
{
    delete (GenericSolver_T *)solver;
}
