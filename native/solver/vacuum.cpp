#include <cstdlib>
#include <cstdint>
#include <cmath>

#include <vector>

#include <deal.II/base/quadrature_lib.h>
#include <deal.II/base/tensor.h>
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
#include <deal.II/lac/precondition.h>
#include <deal.II/numerics/vector_tools.h>
#include <deal.II/numerics/matrix_tools.h>

#include "base/grids.h"
#include "base/defines.h"
#include "base/nbody.h"

typedef struct VacuumSolver_T *VacuumSolver;

#define CUBE_GRID 0

struct VacuumSolver_T
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

SOLVER_API VacuumSolver create_vacuum_solver()
{
    VacuumSolver_T *p_solver = new VacuumSolver_T{};

    p_solver->grid_type = CUBE_GRID;
    p_solver->grid.cube.width = 1.0;
    p_solver->grid.cube.height = 1.0;
    p_solver->grid.cube.depth = 1.0;
    p_solver->grid.cube.refinement = 0;

    return p_solver;
}

SOLVER_API void run_vacuum_solver(VacuumSolver solver)
{
    using namespace dealii;

    /////////////////////////
    // Grid/Domain //////////
    /////////////////////////

    auto triangulation = Triangulation<3>();

    GridGenerator::hyper_rectangle(triangulation, Point<3>{-1.0, -1.0, -1.0}, Point<3>{1.0, 1.0, 1.0});
    triangulation.refine_global(5);

    auto cache = GridTools::Cache<3, 3>(triangulation);

    ////////////////////////
    // Dofs ////////////////
    ////////////////////////

    const auto degree = 1;

    auto q_formula = QGauss<3>{degree + 1};

    const auto fe = FE_Q<3>(degree);

    auto dof_handler = DoFHandler<3>{triangulation};
    dof_handler.distribute_dofs(fe);

    DoFRenumbering::Cuthill_McKee(dof_handler);

    auto constraints = AffineConstraints<double>();

    DoFTools::make_hanging_node_constraints(dof_handler, constraints);

    constraints.close();

    auto fe_values = FEValues<3>{fe,
                                 q_formula,
                                 update_values | update_gradients | update_JxW_values | update_hessians};

    /////////////////////////
    // Config ///////////////
    /////////////////////////

    const auto n_active_cells = triangulation.n_active_cells();
    const auto n_quadrature_points_per_cell = q_formula.size();

    const auto n_dofs = dof_handler.n_dofs();
    const auto n_dofs_per_cell = fe.n_dofs_per_cell();

    double delta_time = 1.0;
    uint32_t max_iteration = 0;

    //////////////////////////////
    // Spacetime ////////////////
    /////////////////////////////

    // Metric: finite element representation

    auto metric_11 = Vector<double>{n_dofs};
    auto metric_12 = Vector<double>{n_dofs};
    auto metric_13 = Vector<double>{n_dofs};
    auto metric_22 = Vector<double>{n_dofs};
    auto metric_23 = Vector<double>{n_dofs};
    auto metric_33 = Vector<double>{n_dofs};

    auto metric_rhs_11 = Vector<double>{n_dofs};
    auto metric_rhs_12 = Vector<double>{n_dofs};
    auto metric_rhs_13 = Vector<double>{n_dofs};
    auto metric_rhs_22 = Vector<double>{n_dofs};
    auto metric_rhs_23 = Vector<double>{n_dofs};
    auto metric_rhs_33 = Vector<double>{n_dofs};

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

    auto extrinsic_11 = Vector<double>{n_dofs};
    auto extrinsic_12 = Vector<double>{n_dofs};
    auto extrinsic_13 = Vector<double>{n_dofs};
    auto extrinsic_22 = Vector<double>{n_dofs};
    auto extrinsic_23 = Vector<double>{n_dofs};
    auto extrinsic_33 = Vector<double>{n_dofs};

    auto extrinsic_rhs_11 = Vector<double>{n_dofs};
    auto extrinsic_rhs_12 = Vector<double>{n_dofs};
    auto extrinsic_rhs_13 = Vector<double>{n_dofs};
    auto extrinsic_rhs_22 = Vector<double>{n_dofs};
    auto extrinsic_rhs_23 = Vector<double>{n_dofs};
    auto extrinsic_rhs_33 = Vector<double>{n_dofs};

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

    auto lapse = Vector<double>{n_dofs};

    auto lapse_rhs = Vector<double>{n_dofs};

    auto lapse_values = std::vector<double>(n_quadrature_points_per_cell);

    auto lapse_hessians = std::vector<Tensor<2, 3, double>>(n_quadrature_points_per_cell);

    auto cell_lapse_rhs = Vector<double>(n_dofs_per_cell);

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

    //////////////////////////
    // Time Loop /////////////
    //////////////////////////

    // Local Dof indices

    std::vector<types::global_dof_index> local_dof_indices(n_dofs_per_cell);

    // Boundry

    auto boundary_values = std::map<types::global_dof_index, double>{};

    // Nbodies

    auto nbody_points = std::vector<Point<3, double>>();

    for (uint32_t i = 0; i < max_iteration; i++)
    {

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
            SolverControl solver_control(100, 1e-12);
            SolverCG<Vector<double>> cg(solver_control);

            cg.solve(shape_matrix, lapse, lapse_rhs, PreconditionIdentity{});
        }

        /////////////////////////////////////////
        // Main Evolution //////////////////////
        /////////////////////////////////////////

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
#define RICCI(i, j, k) CONCAT(ricci_, INDICES(i, j))

                // Compute extrinsic trace

                auto extrinsic_trace = INV_METRIC_VALUE(1, 1) * EXTRINSIC_VALUE(1, 1) + 2.0 * INV_METRIC_VALUE(1, 2) * EXTRINSIC_VALUE(1, 2) + 2.0 * INV_METRIC_VALUE(1, 3) * EXTRINSIC_VALUE(1, 3) +
                                       INV_METRIC_VALUE(2, 2) * EXTRINSIC_VALUE(2, 2) + 2.0 * INV_METRIC_VALUE(2, 3) * EXTRINSIC_VALUE(2, 3) +
                                       INV_METRIC_VALUE(3, 3) * EXTRINSIC_VALUE(3, 3);

#define EXTRINSIC_TRACE extrinsic_trace

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
            metric_rhs_11 *= delta_time;
            metric_rhs_12 *= delta_time;
            metric_rhs_13 *= delta_time;
            metric_rhs_22 *= delta_time;
            metric_rhs_23 *= delta_time;
            metric_rhs_33 *= delta_time;
            // Add previous metric for final rhs
            shape_matrix.vmult_add(metric_rhs_11, metric_11);
            shape_matrix.vmult_add(metric_rhs_12, metric_12);
            shape_matrix.vmult_add(metric_rhs_13, metric_13);
            shape_matrix.vmult_add(metric_rhs_22, metric_22);
            shape_matrix.vmult_add(metric_rhs_23, metric_23);
            shape_matrix.vmult_add(metric_rhs_33, metric_33);

            SolverControl solver_control(100, 1e-12);
            SolverCG<Vector<double>> cg(solver_control);

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

            solve_system(metric_11, metric_rhs_11, ConstantFunction<3, double>(1.0));
            solve_system(metric_12, metric_rhs_12, ZeroFunction<3, double>());
            solve_system(metric_13, metric_rhs_13, ZeroFunction<3, double>());
            solve_system(metric_22, metric_rhs_22, ConstantFunction<3, double>(1.0));
            solve_system(metric_23, metric_rhs_23, ZeroFunction<3, double>());
            solve_system(metric_33, metric_rhs_33, ConstantFunction<3, double>(1.0));
        }

        {
            // Scale extrinsic by delta time
            extrinsic_rhs_11 *= delta_time;
            extrinsic_rhs_12 *= delta_time;
            extrinsic_rhs_13 *= delta_time;
            extrinsic_rhs_22 *= delta_time;
            extrinsic_rhs_23 *= delta_time;
            extrinsic_rhs_33 *= delta_time;
            // Add previous extrinsic for final rhs
            shape_matrix.vmult_add(extrinsic_rhs_11, extrinsic_11);
            shape_matrix.vmult_add(extrinsic_rhs_12, extrinsic_12);
            shape_matrix.vmult_add(extrinsic_rhs_13, extrinsic_13);
            shape_matrix.vmult_add(extrinsic_rhs_22, extrinsic_22);
            shape_matrix.vmult_add(extrinsic_rhs_23, extrinsic_23);
            shape_matrix.vmult_add(extrinsic_rhs_33, extrinsic_33);

            SolverControl solver_control(100, 1e-12);
            SolverCG<Vector<double>> cg(solver_control);

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
}

SOLVER_API void destroy_vacuum_solver(VacuumSolver solver)
{
    delete (VacuumSolver_T *)solver;
}
