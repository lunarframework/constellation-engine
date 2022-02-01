#include <cstdlib>
#include <cstdint>
#include <cmath>

#include <string>
#include <vector>
#include <iostream>
#include <fstream>
#include <exception>

#include <deal.II/base/quadrature_lib.h>
#include <deal.II/base/tensor.h>
#include <deal.II/base/function.h>
#include <deal.II/grid/tria.h>
#include <deal.II/grid/grid_generator.h>
#include <deal.II/grid/grid_tools.h>
#include <deal.II/grid/grid_tools_cache.h>
#include <deal.II/dofs/dof_handler.h>
#include <deal.II/dofs/dof_tools.h>
#include <deal.II/dofs/dof_renumbering.h>
#include <deal.II/fe/fe_q.h>
#include <deal.II/fe/fe_values.h>
#include <deal.II/fe/fe_system.h>
#include <deal.II/lac/affine_constraints.h>
#include <deal.II/lac/vector.h>
#include <deal.II/lac/full_matrix.h>
#include <deal.II/lac/sparse_matrix.h>
#include <deal.II/lac/dynamic_sparsity_pattern.h>
#include <deal.II/lac/solver_cg.h>
#include <deal.II/lac/solver_minres.h>
#include <deal.II/lac/precondition.h>
#include <deal.II/numerics/vector_tools.h>
#include <deal.II/numerics/matrix_tools.h>
#include <deal.II/numerics/data_out.h>
#include <deal.II/numerics/fe_field_function.h>

#include "base/defines.h"
#include "context/context.hpp"
#include "context/mesh.hpp"

#include <functional>

#define CUBE_GRID 0

struct RingSolver
{
    Context *p_context;
};

struct Domain
{
    unsigned int refinement;
    double outer_ratio;
};

struct Accuracy
{
    unsigned int lapse_iterations;
    double lapse_residual;

    unsigned int metric_iterations;
    double metric_residual;

    unsigned int extrinsic_iterations;
    double extrinsic_residual;
};

SOLVER_API RingSolver *ring_solver_create(Context *p_context)
{
    RingSolver *p_solver = new RingSolver();
    p_solver->p_context = p_context;

    return p_solver;
}

SOLVER_API double ring_solver_run(RingSolver *p_solver, double delta, unsigned int steps, Domain domain, Accuracy accuracy, double mass)
{
    try
    {

        std::cout << "Running ring solver with delta: " << delta << "; steps: " << steps << "; lapse_iterations: " << accuracy.lapse_iterations << "; lapse residual: " << accuracy.lapse_residual << std::endl;

        using namespace dealii;

        const auto origin = Point<3>(0.0, 0.0, 0.0);

        const auto g = p_solver->p_context->gravitational_constant;
        const auto c = p_solver->p_context->speed_of_light;
        const auto c_sq = c * c;
        const auto g_over_c4 = g / (c_sq * c_sq);

        auto inner_radius = 2.0 * g * mass / c_sq;

        auto isotropic = [mass, origin, g, c_sq](Point<3> point)
        {
            auto term = 1.0 + mass * g / (2.0 * c_sq * point.distance(origin));
            return term * term * term * term;
        };

        auto isotropic_schwarzchild = ScalarFunctionFromFunctionObject<3, double>(std::function(isotropic));

        /////////////////////////
        // Grid/Domain //////////
        /////////////////////////

        auto triangulation = Triangulation<3>();

        GridGenerator::hyper_shell(triangulation, origin, inner_radius, domain.outer_ratio * inner_radius);
        triangulation.refine_global(domain.refinement);

        std::cout << "Built Triangulation with " << triangulation.n_vertices() << " vertices" << std::endl;

        // auto cache = GridTools::Cache<3, 3>(triangulation);

        ////////////////////////
        // Dofs ////////////////
        ////////////////////////

        const auto fe = FE_Q<3>(2);

        auto q_formula = QGauss<3>{fe.degree + 1};

        auto dof_handler = DoFHandler<3>{triangulation};
        dof_handler.distribute_dofs(fe);

        DoFRenumbering::Cuthill_McKee(dof_handler);

        auto constraints = AffineConstraints<double>();

        DoFTools::make_hanging_node_constraints(dof_handler, constraints);

        constraints.close();

        auto fe_values = FEValues<3>{fe,
                                     q_formula,
                                     update_values | update_gradients | update_JxW_values | update_hessians | update_quadrature_points};

        std::cout << "Built Dofs" << std::endl;

        /////////////////////////
        // Config ///////////////
        /////////////////////////

        const auto n_active_cells = triangulation.n_active_cells();
        const auto n_quadrature_points_per_cell = q_formula.size();

        const auto n_dofs = dof_handler.n_dofs();
        const auto n_dofs_per_cell = fe.n_dofs_per_cell();

        std::cout << "NDofs " << n_dofs << " N Dofs per Cell " << n_dofs_per_cell << std::endl;

        //////////////////////////////
        // Spacetime ////////////////
        /////////////////////////////

        // Metric: finite element representation

        auto metric_11 = Vector<double>(n_dofs);
        auto metric_12 = Vector<double>(n_dofs);
        auto metric_13 = Vector<double>(n_dofs);
        auto metric_22 = Vector<double>(n_dofs);
        auto metric_23 = Vector<double>(n_dofs);
        auto metric_33 = Vector<double>(n_dofs);

        auto metric_rhs_11 = Vector<double>(n_dofs);
        auto metric_rhs_12 = Vector<double>(n_dofs);
        auto metric_rhs_13 = Vector<double>(n_dofs);
        auto metric_rhs_22 = Vector<double>(n_dofs);
        auto metric_rhs_23 = Vector<double>(n_dofs);
        auto metric_rhs_33 = Vector<double>(n_dofs);

        // Metric: values and gradients

        auto metric_values_11 = std::vector<double>(n_quadrature_points_per_cell);
        auto metric_values_12 = std::vector<double>(n_quadrature_points_per_cell);
        auto metric_values_13 = std::vector<double>(n_quadrature_points_per_cell);
        auto metric_values_22 = std::vector<double>(n_quadrature_points_per_cell);
        auto metric_values_23 = std::vector<double>(n_quadrature_points_per_cell);
        auto metric_values_33 = std::vector<double>(n_quadrature_points_per_cell);

        auto metric_gradients_11 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);
        auto metric_gradients_12 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);
        auto metric_gradients_13 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);
        auto metric_gradients_22 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);
        auto metric_gradients_23 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);
        auto metric_gradients_33 = std::vector<Tensor<1, 3, double>>(n_quadrature_points_per_cell);

        auto metric_hessians_11 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);
        auto metric_hessians_12 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);
        auto metric_hessians_13 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);
        auto metric_hessians_22 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);
        auto metric_hessians_23 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);
        auto metric_hessians_33 = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);

        // Metric: cell

        auto cell_metric_rhs_11 = Vector<double>(n_dofs_per_cell);
        auto cell_metric_rhs_12 = Vector<double>(n_dofs_per_cell);
        auto cell_metric_rhs_13 = Vector<double>(n_dofs_per_cell);
        auto cell_metric_rhs_22 = Vector<double>(n_dofs_per_cell);
        auto cell_metric_rhs_23 = Vector<double>(n_dofs_per_cell);
        auto cell_metric_rhs_33 = Vector<double>(n_dofs_per_cell);

        // Extrinsic Curvature: finite element representation

        auto extrinsic_11 = Vector<double>(n_dofs);
        auto extrinsic_12 = Vector<double>(n_dofs);
        auto extrinsic_13 = Vector<double>(n_dofs);
        auto extrinsic_22 = Vector<double>(n_dofs);
        auto extrinsic_23 = Vector<double>(n_dofs);
        auto extrinsic_33 = Vector<double>(n_dofs);

        auto extrinsic_rhs_11 = Vector<double>(n_dofs);
        auto extrinsic_rhs_12 = Vector<double>(n_dofs);
        auto extrinsic_rhs_13 = Vector<double>(n_dofs);
        auto extrinsic_rhs_22 = Vector<double>(n_dofs);
        auto extrinsic_rhs_23 = Vector<double>(n_dofs);
        auto extrinsic_rhs_33 = Vector<double>(n_dofs);

        // Extrinsic Curvature: values

        auto extrinsic_values_11 = std::vector<double>(n_quadrature_points_per_cell, 0.0);
        auto extrinsic_values_12 = std::vector<double>(n_quadrature_points_per_cell, 0.0);
        auto extrinsic_values_13 = std::vector<double>(n_quadrature_points_per_cell, 0.0);
        auto extrinsic_values_22 = std::vector<double>(n_quadrature_points_per_cell, 0.0);
        auto extrinsic_values_23 = std::vector<double>(n_quadrature_points_per_cell, 0.0);
        auto extrinsic_values_33 = std::vector<double>(n_quadrature_points_per_cell, 0.0);

        // Extrinsic Curvature: cell

        auto cell_extrinsic_rhs_11 = Vector<double>(n_dofs_per_cell);
        auto cell_extrinsic_rhs_12 = Vector<double>(n_dofs_per_cell);
        auto cell_extrinsic_rhs_13 = Vector<double>(n_dofs_per_cell);
        auto cell_extrinsic_rhs_22 = Vector<double>(n_dofs_per_cell);
        auto cell_extrinsic_rhs_23 = Vector<double>(n_dofs_per_cell);
        auto cell_extrinsic_rhs_33 = Vector<double>(n_dofs_per_cell);

        // Lapse

        auto lapse = Vector<double>(n_dofs);

        auto lapse_rhs = Vector<double>(n_dofs);

        auto lapse_values = std::vector<double>(n_quadrature_points_per_cell);

        auto lapse_hessians = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);

        auto cell_lapse_rhs = Vector<double>(n_dofs_per_cell);

        ////////////////////////////////
        // Source Terms ////////////////
        ////////////////////////////////

        auto energy_density = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);

        auto momentum_density_1 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_density_2 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_density_3 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);

        auto momentum_flux_11 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_flux_12 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_flux_13 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_flux_22 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_flux_23 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);
        auto momentum_flux_33 = std::vector<double>(n_active_cells * n_quadrature_points_per_cell, 0.0);

        std::cout << "Allocated Vectors" << std::endl;

        ////////////////////////////////
        // Tensor Defines //////////////
        ////////////////////////////////

#define ORDER_INDICES_11 11
#define ORDER_INDICES_12 12
#define ORDER_INDICES_13 13
#define ORDER_INDICES_21 12
#define ORDER_INDICES_22 22
#define ORDER_INDICES_23 23
#define ORDER_INDICES_31 13
#define ORDER_INDICES_32 23
#define ORDER_INDICES_33 33

#define INDICES(a, b) ORDER_INDICES_##a##b

        /////////////////////////////////
        // Shape Functions //////////////
        /////////////////////////////////

        SparsityPattern sparsity_pattern{};

        {
            DynamicSparsityPattern dsp{n_dofs};
            DoFTools::make_sparsity_pattern(dof_handler, dsp, constraints, false);
            sparsity_pattern.copy_from(dsp);
        }

        SparseMatrix<double> shape_matrix{sparsity_pattern};

        FullMatrix<double> cell_shape_matrix(n_dofs_per_cell, n_dofs_per_cell);

        // Scratch matrix usually copied from shape_matrix
        SparseMatrix<double> system_matrix{sparsity_pattern};

        std::cout << "Built matrices" << std::endl;

        //////////////////////////
        // Time Loop /////////////
        //////////////////////////

        // Local Dof indices

        std::vector<types::global_dof_index> local_dof_indices(n_dofs_per_cell);

        // Boundry

        auto boundary_values = std::map<types::global_dof_index, double>{};

        std::cout << "Constructing Initial Data" << std::endl;

        std::cout << "  Constructing Shape Matrix" << std::endl;

        // Temporarily generate  initial data

        for (const auto &cell : dof_handler.active_cell_iterators())
        {
            fe_values.reinit(cell);
            cell->get_dof_indices(local_dof_indices);

            /////////////////////////////////
            // Shape Matrix /////////////////
            /////////////////////////////////

            cell_shape_matrix = 0;

            for (const unsigned int i : fe_values.dof_indices())
            {
                for (const unsigned int j : fe_values.dof_indices())
                {
                    for (const unsigned int q_index :
                         fe_values.quadrature_point_indices())
                    {
                        cell_shape_matrix(i, j) += fe_values.shape_value(i, q_index) *
                                                   fe_values.shape_value(j, q_index) *
                                                   fe_values.JxW(q_index);
                    }
                }
            }

            constraints.distribute_local_to_global(
                cell_shape_matrix, local_dof_indices, shape_matrix);
        }

        std::cout << "  Constructing Metric and Extrinsic" << std::endl;

        for (const auto &cell : dof_handler.active_cell_iterators())
        {
            fe_values.reinit(cell);
            cell->get_dof_indices(local_dof_indices);

            //////////////////////////////////
            // Spacetime /////////////////////
            //////////////////////////////////

            cell_metric_rhs_11 = 0;
            cell_metric_rhs_12 = 0;
            cell_metric_rhs_13 = 0;
            cell_metric_rhs_22 = 0;
            cell_metric_rhs_23 = 0;
            cell_metric_rhs_33 = 0;

            cell_extrinsic_rhs_11 = 0;
            cell_extrinsic_rhs_12 = 0;
            cell_extrinsic_rhs_13 = 0;
            cell_extrinsic_rhs_22 = 0;
            cell_extrinsic_rhs_23 = 0;
            cell_extrinsic_rhs_33 = 0;

            for (const unsigned int q_index :
                 fe_values.quadrature_point_indices())
            {
                auto coefficient = isotropic_schwarzchild.value(fe_values.quadrature_point(q_index));

                for (const unsigned int i : fe_values.dof_indices())
                {
                    auto integrator = fe_values.shape_value(i, q_index) * fe_values.JxW(q_index);

                    cell_metric_rhs_11(i) += coefficient * integrator;
                    cell_metric_rhs_12(i) += 0.0 * integrator;
                    cell_metric_rhs_13(i) += 0.0 * integrator;
                    cell_metric_rhs_22(i) += coefficient * integrator;
                    cell_metric_rhs_23(i) += 0.0 * integrator;
                    cell_metric_rhs_33(i) += coefficient * integrator;

                    cell_extrinsic_rhs_11(i) += 0.0 * integrator;
                    cell_extrinsic_rhs_12(i) += 0.0 * integrator;
                    cell_extrinsic_rhs_13(i) += 0.0 * integrator;
                    cell_extrinsic_rhs_22(i) += 0.0 * integrator;
                    cell_extrinsic_rhs_23(i) += 0.0 * integrator;
                    cell_extrinsic_rhs_33(i) += 0.0 * integrator;
                }
            }

            // std::cout << "Distributing" << std::endl;

            // std::cout << "local_dof_indices size" << local_dof_indices.size() << std::endl;
            // std::cout << "cell_metric_rhs_11 size" << cell_metric_rhs_11.size() << std::endl;
            // std::cout << "metric_rhs_11 size" << metric_rhs_11.size() << std::endl;

            // #define DISTRIBUTE_LOCAL_TO_GLOBAL(local, global)         \
//     for (const unsigned int i : fe_values.dof_indices())  \
//     {                                                     \
//         ##global##(local_dof_indices[i]) += ##local##(i); \
//     }

            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_11, metric_rhs_11);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_12, metric_rhs_12);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_13, metric_rhs_13);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_22, metric_rhs_22);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_23, metric_rhs_23);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_metric_rhs_33, metric_rhs_33);

            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_11, extrinsic_rhs_11);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_12, extrinsic_rhs_12);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_13, extrinsic_rhs_13);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_22, extrinsic_rhs_22);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_23, extrinsic_rhs_23);
            //         DISTRIBUTE_LOCAL_TO_GLOBAL(cell_extrinsic_rhs_33, extrinsic_rhs_33);

            constraints.distribute_local_to_global(cell_metric_rhs_11, local_dof_indices, metric_rhs_11);
            constraints.distribute_local_to_global(cell_metric_rhs_12, local_dof_indices, metric_rhs_12);
            constraints.distribute_local_to_global(cell_metric_rhs_13, local_dof_indices, metric_rhs_13);
            constraints.distribute_local_to_global(cell_metric_rhs_22, local_dof_indices, metric_rhs_22);
            constraints.distribute_local_to_global(cell_metric_rhs_23, local_dof_indices, metric_rhs_23);
            constraints.distribute_local_to_global(cell_metric_rhs_33, local_dof_indices, metric_rhs_33);

            constraints.distribute_local_to_global(cell_extrinsic_rhs_11, local_dof_indices, extrinsic_rhs_11);
            constraints.distribute_local_to_global(cell_extrinsic_rhs_12, local_dof_indices, extrinsic_rhs_12);
            constraints.distribute_local_to_global(cell_extrinsic_rhs_13, local_dof_indices, extrinsic_rhs_13);
            constraints.distribute_local_to_global(cell_extrinsic_rhs_22, local_dof_indices, extrinsic_rhs_22);
            constraints.distribute_local_to_global(cell_extrinsic_rhs_23, local_dof_indices, extrinsic_rhs_23);
            constraints.distribute_local_to_global(cell_extrinsic_rhs_33, local_dof_indices, extrinsic_rhs_33);
        }

        std::cout << "  Solving Metric and Extrinsic" << std::endl;

        {
            SolverControl solver_control(accuracy.metric_iterations, accuracy.metric_residual);
            SolverCG<Vector<double>> cg(solver_control);
            // SolverMinRes<Vector<double>> min_res(solver_control);

            auto solve_system = [&](auto &metric, auto &metric_rhs, auto &boundary)
            {
                boundary_values.clear();
                VectorTools::interpolate_boundary_values(dof_handler,
                                                         0,
                                                         boundary,
                                                         boundary_values);

                system_matrix.copy_from(shape_matrix);
                MatrixTools::apply_boundary_values(boundary_values,
                                                   system_matrix,
                                                   metric,
                                                   metric_rhs);

                cg.solve(system_matrix, metric, metric_rhs, PreconditionIdentity{});

                constraints.distribute(metric);
            };

            solve_system(metric_11, metric_rhs_11, isotropic_schwarzchild);
            solve_system(metric_12, metric_rhs_12, ZeroFunction<3, double>());
            solve_system(metric_13, metric_rhs_13, ZeroFunction<3, double>());
            solve_system(metric_22, metric_rhs_22, isotropic_schwarzchild);
            solve_system(metric_23, metric_rhs_23, ZeroFunction<3, double>());
            solve_system(metric_33, metric_rhs_33, isotropic_schwarzchild);
        }

        {
            SolverControl solver_control(accuracy.extrinsic_iterations, accuracy.extrinsic_residual);
            SolverCG<Vector<double>> cg(solver_control);
            // SolverMinRes<Vector<double>> min_res(solver_control);

            auto solve_system = [&](auto &metric, auto &metric_rhs, auto &boundary)
            {
                boundary_values.clear();
                VectorTools::interpolate_boundary_values(dof_handler,
                                                         0,
                                                         boundary,
                                                         boundary_values);

                system_matrix.copy_from(shape_matrix);
                MatrixTools::apply_boundary_values(boundary_values,
                                                   system_matrix,
                                                   metric,
                                                   metric_rhs);

                cg.solve(system_matrix, metric, metric_rhs, PreconditionIdentity{});

                constraints.distribute(metric);
            };

            solve_system(extrinsic_11, extrinsic_rhs_11, ZeroFunction<3, double>(1));
            solve_system(extrinsic_12, extrinsic_rhs_12, ZeroFunction<3, double>(1));
            solve_system(extrinsic_13, extrinsic_rhs_13, ZeroFunction<3, double>(1));
            solve_system(extrinsic_22, extrinsic_rhs_22, ZeroFunction<3, double>(1));
            solve_system(extrinsic_23, extrinsic_rhs_23, ZeroFunction<3, double>(1));
            solve_system(extrinsic_33, extrinsic_rhs_33, ZeroFunction<3, double>(1));
        }

        std::cout << "Constructed Initial Data" << std::endl;

        // Nbodies

        // auto nbody_points = std::vector<Point<3, double>>();

        for (uint32_t i = 0; i < steps; i++)
        {

            std::cout << "Running step " << i << std::endl;

            ///////////////////////////////////////
            // Shape Matrix ///////////////////////
            ///////////////////////////////////////

            for (const auto &cell : dof_handler.active_cell_iterators())
            {
                fe_values.reinit(cell);
                cell->get_dof_indices(local_dof_indices);

                /////////////////////////////////
                // Shape Matrix /////////////////
                /////////////////////////////////

                cell_shape_matrix = 0;

                for (const unsigned int i : fe_values.dof_indices())
                {
                    for (const unsigned int j : fe_values.dof_indices())
                    {
                        for (const unsigned int q_index :
                             fe_values.quadrature_point_indices())
                        {
                            cell_shape_matrix(i, j) += fe_values.shape_value(i, q_index) *
                                                       fe_values.shape_value(j, q_index) *
                                                       fe_values.JxW(q_index);
                        }
                    }
                }

                constraints.distribute_local_to_global(
                    cell_shape_matrix, local_dof_indices, shape_matrix);
            }

            ///////////////////////////////////////
            // Pre processing /////////////////////
            ///////////////////////////////////////

            for (const auto &cell : dof_handler.active_cell_iterators())
            {
                fe_values.reinit(cell);
                cell->get_dof_indices(local_dof_indices);

                cell_lapse_rhs = 0;

                fe_values.get_function_values(metric_11, metric_values_11);
                fe_values.get_function_values(metric_12, metric_values_12);
                fe_values.get_function_values(metric_13, metric_values_13);
                fe_values.get_function_values(metric_22, metric_values_22);
                fe_values.get_function_values(metric_23, metric_values_23);
                fe_values.get_function_values(metric_33, metric_values_33);

                for (const auto q_index : fe_values.quadrature_point_indices())
                {
#define METRIC_VALUE(i, j) CONCAT(CONCAT(metric_values_, INDICES(i, j)), [q_index])

                    auto metric_det = METRIC_VALUE(1, 1) * (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(3, 2)) -
                                      METRIC_VALUE(1, 2) * (METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(1, 3)) +
                                      METRIC_VALUE(1, 3) * (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(2, 2) * METRIC_VALUE(1, 3));

                    for (const auto i : fe_values.dof_indices())
                    {
                        auto integrator = fe_values.shape_value(i, q_index) * fe_values.JxW(q_index);
                        cell_lapse_rhs(i) += (1.0 + log(metric_det)) * integrator;
                    }
                }

                constraints.distribute_local_to_global(
                    cell_lapse_rhs, local_dof_indices, lapse_rhs);
            }

            {
                SolverControl solver_control(accuracy.lapse_iterations, accuracy.lapse_residual);
                SolverCG<Vector<double>> cg(solver_control);
                // SolverMinRes<Vector<double>> min_res(solver_control);

                cg.solve(shape_matrix, lapse, lapse_rhs, PreconditionIdentity{});
            }

            //////////////////////////////////
            // Source Terms //////////////////
            //////////////////////////////////

            // {
            //     nbody_points.clear();
            //     nbody_points.reserve(p_solver->nbodies.size());

            //     for (auto &nbody : p_solver->nbodies)
            //     {
            //         nbody_points.push_back(Point<3, double>(nbody.x, nbody.y, nbody.z));
            //     }

            //     auto [cells, qpoints, indices] = GridTools::compute_point_locations(cache, nbody_points);

            //     for (int cell_index = 0; cell_index < cells.size(); cell_index++)
            //     {
            //         auto global_cell_index = cells[cell_index]->index();
            //         for (int point_index = 0; point_index < qpoints[cell_index].size(); point_index++)
            //         {
            //             for (const auto q_index : fe_values.quadrature_point_indices())
            //             {
            //                 auto vector_index = global_cell_index * n_quadrature_points_per_cell + q_index;

            //                 energy_density[vector_index] = 0.0;
            //                 momentum_density_1[vector_index] = 0.0;
            //                 momentum_density_2[vector_index] = 0.0;
            //                 momentum_density_3[vector_index] = 0.0;
            //                 momentum_flux_11[vector_index] = 0.0;
            //                 momentum_flux_12[vector_index] = 0.0;
            //                 momentum_flux_13[vector_index] = 0.0;
            //                 momentum_flux_22[vector_index] = 0.0;
            //                 momentum_flux_23[vector_index] = 0.0;
            //                 momentum_flux_33[vector_index] = 0.0;
            //             }
            //         }
            //     }
            // }

            /////////////////////////////////////////
            // Main Evolution //////////////////////
            /////////////////////////////////////////

            const auto g_over_c4 = g / (c_sq * c_sq);

            for (const auto &cell : dof_handler.active_cell_iterators())
            {
                fe_values.reinit(cell);
                cell->get_dof_indices(local_dof_indices);

                //////////////////////////////////
                // Spacetime /////////////////////
                //////////////////////////////////

                cell_metric_rhs_11 = 0;
                cell_metric_rhs_12 = 0;
                cell_metric_rhs_13 = 0;
                cell_metric_rhs_22 = 0;
                cell_metric_rhs_23 = 0;
                cell_metric_rhs_33 = 0;

                cell_extrinsic_rhs_11 = 0;
                cell_extrinsic_rhs_12 = 0;
                cell_extrinsic_rhs_13 = 0;
                cell_extrinsic_rhs_22 = 0;
                cell_extrinsic_rhs_23 = 0;
                cell_extrinsic_rhs_33 = 0;

                fe_values.get_function_values(metric_11, metric_values_11);
                fe_values.get_function_values(metric_12, metric_values_12);
                fe_values.get_function_values(metric_13, metric_values_13);
                fe_values.get_function_values(metric_22, metric_values_22);
                fe_values.get_function_values(metric_23, metric_values_23);
                fe_values.get_function_values(metric_33, metric_values_33);

                fe_values.get_function_gradients(metric_11, metric_gradients_11);
                fe_values.get_function_gradients(metric_12, metric_gradients_12);
                fe_values.get_function_gradients(metric_13, metric_gradients_13);
                fe_values.get_function_gradients(metric_22, metric_gradients_22);
                fe_values.get_function_gradients(metric_23, metric_gradients_23);
                fe_values.get_function_gradients(metric_33, metric_gradients_33);

                fe_values.get_function_hessians(metric_11, metric_hessians_11);
                fe_values.get_function_hessians(metric_12, metric_hessians_12);
                fe_values.get_function_hessians(metric_13, metric_hessians_13);
                fe_values.get_function_hessians(metric_22, metric_hessians_22);
                fe_values.get_function_hessians(metric_23, metric_hessians_23);
                fe_values.get_function_hessians(metric_33, metric_hessians_33);

                fe_values.get_function_values(extrinsic_11, extrinsic_values_11);
                fe_values.get_function_values(extrinsic_12, extrinsic_values_12);
                fe_values.get_function_values(extrinsic_13, extrinsic_values_13);
                fe_values.get_function_values(extrinsic_22, extrinsic_values_22);
                fe_values.get_function_values(extrinsic_23, extrinsic_values_23);
                fe_values.get_function_values(extrinsic_33, extrinsic_values_33);

                fe_values.get_function_values(lapse, lapse_values);
                fe_values.get_function_hessians(lapse, lapse_hessians);

                const auto q_index_offset = n_quadrature_points_per_cell * cell->index();

                for (const unsigned int q_index :
                     fe_values.quadrature_point_indices())
                {
                    // Defines to access metric and extrinsic
#define METRIC_VALUE(i, j) CONCAT(CONCAT(metric_values_, INDICES(i, j)), [q_index])
#define METRIC_GRADIENT(i, j, a) CONCAT(CONCAT(metric_gradients_, INDICES(i, j)), [q_index][##a## - 1])
#define METRIC_HESSIAN(i, j, a, b) CONCAT(CONCAT(metric_hessians_, INDICES(i, j)), [q_index][TableIndices<2>(a - 1, b - 1)])
#define EXTRINSIC_VALUE(i, j) CONCAT(CONCAT(extrinsic_values_, INDICES(i, j)), [q_index])
#define LAPSE_VALUE lapse_values[q_index]
#define LAPSE_HESSIAN(i, j) lapse_hessians[q_index][TableIndices<2>(i - 1, j - 1)]

                    // Compute determinate of metric
                    auto metric_det = METRIC_VALUE(1, 1) * (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(3, 2)) -
                                      METRIC_VALUE(1, 2) * (METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(1, 3)) +
                                      METRIC_VALUE(1, 3) * (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(2, 2) * METRIC_VALUE(1, 3));

                    auto inv_metric_det = 1.0 / metric_det;

                    auto inv_metric_11 = (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(2, 3)) * inv_metric_det;
                    auto inv_metric_12 = (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3)) * inv_metric_det;
                    auto inv_metric_13 = (METRIC_VALUE(1, 2) * METRIC_VALUE(2, 3) - METRIC_VALUE(1, 3) * METRIC_VALUE(2, 2)) * inv_metric_det;
                    auto inv_metric_22 = (METRIC_VALUE(1, 1) * METRIC_VALUE(3, 3) - METRIC_VALUE(1, 3) * METRIC_VALUE(1, 3)) * inv_metric_det;
                    auto inv_metric_23 = (METRIC_VALUE(1, 2) * METRIC_VALUE(1, 3) - METRIC_VALUE(1, 1) * METRIC_VALUE(2, 3)) * inv_metric_det;
                    auto inv_metric_33 = (METRIC_VALUE(1, 1) * METRIC_VALUE(2, 2) - METRIC_VALUE(1, 2) * METRIC_VALUE(1, 2)) * inv_metric_det;

                    // Define to access inverse metric
#define INV_METRIC_VALUE(i, j) CONCAT(inv_metric_, INDICES(i, j))

                    // Compute connections
#define CONNECTION_TERM(i, j, k, l) 0.5 * INV_METRIC_VALUE(i, l) * (METRIC_GRADIENT(l, j, k) + METRIC_GRADIENT(l, k, j) - METRIC_GRADIENT(j, k, l))
#define COMPUTE_CONNECTION(i, j, k) CONNECTION_TERM(i, j, k, 1) + CONNECTION_TERM(i, j, k, 2) + CONNECTION_TERM(i, j, k, 3)

                    auto connection_11_1 = COMPUTE_CONNECTION(1, 1, 1);
                    auto connection_11_2 = COMPUTE_CONNECTION(1, 1, 2);
                    auto connection_11_3 = COMPUTE_CONNECTION(1, 1, 3);

                    auto connection_12_1 = COMPUTE_CONNECTION(1, 2, 1);
                    auto connection_12_2 = COMPUTE_CONNECTION(1, 2, 2);
                    auto connection_12_3 = COMPUTE_CONNECTION(1, 2, 3);

                    auto connection_13_1 = COMPUTE_CONNECTION(1, 3, 1);
                    auto connection_13_2 = COMPUTE_CONNECTION(1, 3, 2);
                    auto connection_13_3 = COMPUTE_CONNECTION(1, 3, 3);

                    auto connection_22_1 = COMPUTE_CONNECTION(2, 2, 1);
                    auto connection_22_2 = COMPUTE_CONNECTION(2, 2, 2);
                    auto connection_22_3 = COMPUTE_CONNECTION(2, 2, 3);

                    auto connection_23_1 = COMPUTE_CONNECTION(2, 3, 1);
                    auto connection_23_2 = COMPUTE_CONNECTION(2, 3, 2);
                    auto connection_23_3 = COMPUTE_CONNECTION(2, 3, 3);

                    auto connection_33_1 = COMPUTE_CONNECTION(3, 3, 1);
                    auto connection_33_2 = COMPUTE_CONNECTION(3, 3, 2);
                    auto connection_33_3 = COMPUTE_CONNECTION(3, 3, 3);

                    // Define to access connection
#define CONNECTION(i, j, k) CONCAT(CONCAT(connection_, INDICES(i, j)), _##k)

                    // Compute Ricci scaler
#define RICCI_2ND_DER_TERM(i, j, k, l) 0.5 * INV_METRIC_VALUE(k, l) * (METRIC_HESSIAN(k, j, i, l) + METRIC_HESSIAN(i, l, k, j) - METRIC_HESSIAN(k, l, i, j) - METRIC_HESSIAN(i, j, k, l))
#define RICCI_CONNECTION_TERM(i, j, k, l) CONNECTION(i, j, k) * CONNECTION(k, l, l) - CONNECTION(i, l, k) - CONNECTION(j, k, l)
#define COMPUTE_RICCI(i, j) RICCI_2ND_DER_TERM(i, j, 1, 1) + 2.0 * RICCI_2ND_DER_TERM(i, j, 1, 2) + 2.0 * RICCI_2ND_DER_TERM(i, j, 1, 3) +  \
                                RICCI_2ND_DER_TERM(i, j, 2, 2) + 2.0 * RICCI_2ND_DER_TERM(i, j, 2, 3) +                                     \
                                RICCI_2ND_DER_TERM(i, j, 3, 3) +                                                                            \
                                RICCI_CONNECTION_TERM(i, j, 1, 1) + RICCI_CONNECTION_TERM(i, j, 1, 2) + RICCI_CONNECTION_TERM(i, j, 1, 3) + \
                                RICCI_CONNECTION_TERM(i, j, 2, 1) + RICCI_CONNECTION_TERM(i, j, 2, 2) + RICCI_CONNECTION_TERM(i, j, 2, 3) + \
                                RICCI_CONNECTION_TERM(i, j, 3, 1) + RICCI_CONNECTION_TERM(i, j, 3, 2) + RICCI_CONNECTION_TERM(i, j, 3, 3)

                    auto ricci_11 = COMPUTE_RICCI(1, 1);
                    auto ricci_12 = COMPUTE_RICCI(1, 2);
                    auto ricci_13 = COMPUTE_RICCI(1, 3);
                    auto ricci_22 = COMPUTE_RICCI(2, 2);
                    auto ricci_23 = COMPUTE_RICCI(2, 3);
                    auto ricci_33 = COMPUTE_RICCI(3, 3);

                    // Define to access ricci
#define RICCI(i, j) CONCAT(ricci_, INDICES(i, j))

                    // Compute extrinsic trace

                    auto extrinsic_trace = INV_METRIC_VALUE(1, 1) * EXTRINSIC_VALUE(1, 1) + 2.0 * INV_METRIC_VALUE(1, 2) * EXTRINSIC_VALUE(1, 2) + 2.0 * INV_METRIC_VALUE(1, 3) * EXTRINSIC_VALUE(1, 3) +
                                           INV_METRIC_VALUE(2, 2) * EXTRINSIC_VALUE(2, 2) + 2.0 * INV_METRIC_VALUE(2, 3) * EXTRINSIC_VALUE(2, 3) +
                                           INV_METRIC_VALUE(3, 3) * EXTRINSIC_VALUE(3, 3);

#define EXTRINSIC_TRACE extrinsic_trace

                    // #define ENERGY_DENSITY_VALUE energy_density[q_index_offset + q_index]
                    // #define MOMENTUM_DENSITY_VALUE(i) momentum_density_##i##[q_index_offset + q_index]
                    // #define MOMENTUM_FLUX_VALUE(a, b) CONCAT(CONCAT(momentum_flux_, INDICES(a, b)), [q_index_offset + q_index])

#define METRIC_RHS(i, j) -2.0 * EXTRINSIC_VALUE(i, j) * LAPSE_VALUE

                    auto metric_rhs_11 = METRIC_RHS(1, 1);
                    auto metric_rhs_12 = METRIC_RHS(1, 2);
                    auto metric_rhs_13 = METRIC_RHS(1, 3);
                    auto metric_rhs_22 = METRIC_RHS(2, 2);
                    auto metric_rhs_23 = METRIC_RHS(2, 3);
                    auto metric_rhs_33 = METRIC_RHS(3, 3);

                    // Compute kext RHS's

#define EXTRINSIC_VALUE_BY_TRACE(i, j) EXTRINSIC_TRACE *EXTRINSIC_VALUE(i, j)
#define EXTRINSIC_INNER_PRODUCT_TERM(i, j, k, l) EXTRINSIC_VALUE(i, k) * INV_METRIC_VALUE(k, l) * EXTRINSIC_VALUE(l, j)
#define EXTRINSIC_INNER_PRODUCT(i, j) EXTRINSIC_INNER_PRODUCT_TERM(i, j, 1, 1) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 1, 2) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 1, 3) +     \
                                          EXTRINSIC_INNER_PRODUCT_TERM(i, j, 2, 1) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 2, 2) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 3, 3) + \
                                          EXTRINSIC_INNER_PRODUCT_TERM(i, j, 3, 1) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 3, 2) + EXTRINSIC_INNER_PRODUCT_TERM(i, j, 3, 3)

#define EXTRINSIC_RHS(i, j) LAPSE_VALUE *(RICCI(i, j) - 2.0 * EXTRINSIC_INNER_PRODUCT(i, j) + EXTRINSIC_VALUE_BY_TRACE(i, j)) - LAPSE_HESSIAN(i, j)

                    auto extrinsic_rhs_11 = EXTRINSIC_RHS(1, 1);
                    auto extrinsic_rhs_12 = EXTRINSIC_RHS(1, 2);
                    auto extrinsic_rhs_13 = EXTRINSIC_RHS(1, 3);
                    auto extrinsic_rhs_22 = EXTRINSIC_RHS(2, 2);
                    auto extrinsic_rhs_23 = EXTRINSIC_RHS(2, 3);
                    auto extrinsic_rhs_33 = EXTRINSIC_RHS(3, 3);

                    for (const unsigned int i : fe_values.dof_indices())
                    {
                        // Index of which element of the shape functionis not 0.0
                        const auto component_i =
                            fe.system_to_component_index(i).first;

                        auto integrator = fe_values.shape_value(i, q_index) * fe_values.JxW(q_index);

                        cell_metric_rhs_11(i) += metric_rhs_11 * integrator;
                        cell_metric_rhs_12(i) += metric_rhs_12 * integrator;
                        cell_metric_rhs_13(i) += metric_rhs_13 * integrator;
                        cell_metric_rhs_22(i) += metric_rhs_22 * integrator;
                        cell_metric_rhs_23(i) += metric_rhs_23 * integrator;
                        cell_metric_rhs_33(i) += metric_rhs_33 * integrator;

                        cell_extrinsic_rhs_11(i) += extrinsic_rhs_11 * integrator;
                        cell_extrinsic_rhs_12(i) += extrinsic_rhs_12 * integrator;
                        cell_extrinsic_rhs_13(i) += extrinsic_rhs_13 * integrator;
                        cell_extrinsic_rhs_22(i) += extrinsic_rhs_22 * integrator;
                        cell_extrinsic_rhs_23(i) += extrinsic_rhs_23 * integrator;
                        cell_extrinsic_rhs_33(i) += extrinsic_rhs_33 * integrator;
                    }
                }

                constraints.distribute_local_to_global(cell_metric_rhs_11, local_dof_indices, metric_rhs_11);
                constraints.distribute_local_to_global(cell_metric_rhs_12, local_dof_indices, metric_rhs_12);
                constraints.distribute_local_to_global(cell_metric_rhs_13, local_dof_indices, metric_rhs_13);
                constraints.distribute_local_to_global(cell_metric_rhs_22, local_dof_indices, metric_rhs_22);
                constraints.distribute_local_to_global(cell_metric_rhs_23, local_dof_indices, metric_rhs_23);
                constraints.distribute_local_to_global(cell_metric_rhs_33, local_dof_indices, metric_rhs_33);

                constraints.distribute_local_to_global(cell_extrinsic_rhs_11, local_dof_indices, extrinsic_rhs_11);
                constraints.distribute_local_to_global(cell_extrinsic_rhs_12, local_dof_indices, extrinsic_rhs_12);
                constraints.distribute_local_to_global(cell_extrinsic_rhs_13, local_dof_indices, extrinsic_rhs_13);
                constraints.distribute_local_to_global(cell_extrinsic_rhs_22, local_dof_indices, extrinsic_rhs_22);
                constraints.distribute_local_to_global(cell_extrinsic_rhs_23, local_dof_indices, extrinsic_rhs_23);
                constraints.distribute_local_to_global(cell_extrinsic_rhs_33, local_dof_indices, extrinsic_rhs_33);
            }

            {
                // Scale metric by delta time
                metric_rhs_11 *= delta;
                metric_rhs_12 *= delta;
                metric_rhs_13 *= delta;
                metric_rhs_22 *= delta;
                metric_rhs_23 *= delta;
                metric_rhs_33 *= delta;
                // Add previous metric for final rhs
                shape_matrix.vmult_add(metric_rhs_11, metric_11);
                shape_matrix.vmult_add(metric_rhs_12, metric_12);
                shape_matrix.vmult_add(metric_rhs_13, metric_13);
                shape_matrix.vmult_add(metric_rhs_22, metric_22);
                shape_matrix.vmult_add(metric_rhs_23, metric_23);
                shape_matrix.vmult_add(metric_rhs_33, metric_33);

                SolverControl solver_control(accuracy.metric_iterations, accuracy.metric_residual);
                SolverCG<Vector<double>> cg(solver_control);
                // SolverMinRes<Vector<double>> min_res(solver_control);

                auto solve_system = [&](auto &metric, auto &metric_rhs, auto &boundary)
                {
                    boundary_values.clear();
                    VectorTools::interpolate_boundary_values(dof_handler,
                                                             0,
                                                             boundary,
                                                             boundary_values);

                    system_matrix.copy_from(shape_matrix);
                    MatrixTools::apply_boundary_values(boundary_values,
                                                       system_matrix,
                                                       metric,
                                                       metric_rhs);

                    cg.solve(system_matrix, metric, metric_rhs, PreconditionIdentity{});

                    constraints.distribute(metric);
                };

                solve_system(metric_11, metric_rhs_11, isotropic_schwarzchild);
                solve_system(metric_12, metric_rhs_12, ZeroFunction<3, double>());
                solve_system(metric_13, metric_rhs_13, ZeroFunction<3, double>());
                solve_system(metric_22, metric_rhs_22, isotropic_schwarzchild);
                solve_system(metric_23, metric_rhs_23, ZeroFunction<3, double>());
                solve_system(metric_33, metric_rhs_33, isotropic_schwarzchild);
            }

            {
                // Scale extrinsic by delta time
                extrinsic_rhs_11 *= delta;
                extrinsic_rhs_12 *= delta;
                extrinsic_rhs_13 *= delta;
                extrinsic_rhs_22 *= delta;
                extrinsic_rhs_23 *= delta;
                extrinsic_rhs_33 *= delta;
                // Add previous extrinsic for final rhs
                shape_matrix.vmult_add(extrinsic_rhs_11, extrinsic_11);
                shape_matrix.vmult_add(extrinsic_rhs_12, extrinsic_12);
                shape_matrix.vmult_add(extrinsic_rhs_13, extrinsic_13);
                shape_matrix.vmult_add(extrinsic_rhs_22, extrinsic_22);
                shape_matrix.vmult_add(extrinsic_rhs_23, extrinsic_23);
                shape_matrix.vmult_add(extrinsic_rhs_33, extrinsic_33);

                SolverControl solver_control(accuracy.extrinsic_iterations, accuracy.extrinsic_residual);
                SolverCG<Vector<double>> cg(solver_control);
                // SolverMinRes<Vector<double>> min_res(solver_control);

                auto solve_system = [&](auto &metric, auto &metric_rhs, auto &boundary)
                {
                    boundary_values.clear();
                    VectorTools::interpolate_boundary_values(dof_handler,
                                                             0,
                                                             boundary,
                                                             boundary_values);

                    system_matrix.copy_from(shape_matrix);
                    MatrixTools::apply_boundary_values(boundary_values,
                                                       system_matrix,
                                                       metric,
                                                       metric_rhs);

                    cg.solve(system_matrix, metric, metric_rhs, PreconditionIdentity{});

                    constraints.distribute(metric);
                };

                solve_system(extrinsic_11, extrinsic_rhs_11, ZeroFunction<3, double>(1));
                solve_system(extrinsic_12, extrinsic_rhs_12, ZeroFunction<3, double>(1));
                solve_system(extrinsic_13, extrinsic_rhs_13, ZeroFunction<3, double>(1));
                solve_system(extrinsic_22, extrinsic_rhs_22, ZeroFunction<3, double>(1));
                solve_system(extrinsic_23, extrinsic_rhs_23, ZeroFunction<3, double>(1));
                solve_system(extrinsic_33, extrinsic_rhs_33, ZeroFunction<3, double>(1));
            }
        }

        std::cout << "Finished Time Loop" << std::endl;

        double average = 0.0;

        double standard_deviation = 0.0;

        {
            auto metric_11_function = Functions::FEFieldFunction<3>(dof_handler, metric_11);
            auto metric_22_function = Functions::FEFieldFunction<3>(dof_handler, metric_22);
            auto metric_33_function = Functions::FEFieldFunction<3>(dof_handler, metric_33);

            unsigned int total = 0;

            for (auto &point : triangulation.get_vertices())
            {
                auto prediction = metric_11_function.value(point) * metric_22_function.value(point) * metric_33_function.value(point);

                auto term = isotropic_schwarzchild.value(point);

                auto analytical = term * term * term;

                // std::cout << "Point: " << point << " Prediction: " << prediction << " Analytical: " << analytical << std::endl;

                if (abs((prediction - analytical)) > 0.00001)
                {
                    standard_deviation += (prediction - analytical) * (prediction - analytical);
                    average += abs(prediction - analytical);
                    total += 1;
                }
            }

            standard_deviation /= (double)total;
            standard_deviation = sqrt(standard_deviation);
            average /= (double)total;
        }

        std::cout << "Average " << average << " Standard Deviation " << standard_deviation << std::endl;

        return standard_deviation;
    }
    catch (std::exception &e)
    {
        std::cerr << "Exception occurred:" << std::endl
                  << e.what()
                  << std::endl;
        return 0.0;
    }
}

SOLVER_API void ring_solver_destroy(RingSolver *p_solver)
{
    delete p_solver;
}
