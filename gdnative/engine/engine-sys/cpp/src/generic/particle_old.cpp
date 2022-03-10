#include "mfem.hpp"

#include "export.hpp"
#include "constants.hpp"

#include <vector>
#include <iostream>
#include <exception>

using namespace mfem;

#define CONCAT1(first, second) first##second
#define CONCAT(first, second) CONCAT1(first, second)

#define INDICES_11 0
#define INDICES_12 1
#define INDICES_13 2
#define INDICES_21 1
#define INDICES_22 3
#define INDICES_23 4
#define INDICES_31 2
#define INDICES_32 4
#define INDICES_33 5

#define INDICES1(i, j) CONCAT(INDICES_, CONCAT(i, j))
#define INDICES(i, j) INDICES1(i, j)

#define INDEX_1 0
#define INDEX_2 1
#define INDEX_3 2

#define INDEX1(i) CONCAT(INDEX_, i)
#define INDEX(i) INDEX1(i)

class LapseLFIntegrator : public LinearFormIntegrator
{
protected:
    GridFunction &metric;

    Vector metric_values;

    Vector metric_dof_values;

    Vector shape;
    int oa, ob;

public:
    LapseLFIntegrator(GridFunction &metric, int a = 2, int b = 0)
        : metric(metric), oa(a), ob(b)
    {
    }

    virtual ~LapseLFIntegrator()
    {
    }

    virtual void AssembleRHSElementVect(const FiniteElement &el, ElementTransformation &Tr, Vector &elvect)
    {
        const int vdim = 6;
        const int dof = el.GetDof();

        metric_values.SetSize(vdim);

        shape.SetSize(dof);
        elvect.SetSize(dof);
        elvect = 0.0;

        const IntegrationRule *ir = IntRule;
        if (ir == NULL)
        {
            ir = &IntRules.Get(el.GetGeomType(), oa * el.GetOrder() + ob);
        }

        metric.GetElementDofValues(Tr.ElementNo, metric_dof_values);

        for (int i = 0; i < ir->GetNPoints(); i++)
        {
            const IntegrationPoint &ip = ir->IntPoint(i);
            Tr.SetIntPoint(&ip);

            double weight = Tr.Weight() * ip.weight;

            el.CalcPhysShape(Tr, shape);

            metric_values = 0.0;

            for (int k = 0; k < vdim; k++)
            {
                for (int df = 0; df < dof; df++)
                {
                    metric_values(k) += shape(df) * metric_dof_values(k * dof + df);
                }
            }

#define METRIC_VALUE(i, j) metric_values(##INDICES(i, j)##)

            double metric_det = METRIC_VALUE(1, 1) * (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(3, 2)) -
                                METRIC_VALUE(1, 2) * (METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(1, 3)) +
                                METRIC_VALUE(1, 3) * (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(2, 2) * METRIC_VALUE(1, 3));

            double rhs = 1.0 + std::log(metric_det);

            for (int s = 0; s < dof; s++)
            {
                elvect(s) += weight * rhs * shape(s);
            }
        }
    }
};

class MetricLFIntegrator : public LinearFormIntegrator
{
protected:
    GridFunction &curvature;
    GridFunction &lapse;

    Vector curvature_values;
    double lapse_value;

    Vector shape;
    int oa, ob;

public:
    MetricLFIntegrator(GridFunction &curvature, GridFunction &lapse, int a = 2, int b = 0)
        : curvature(curvature), lapse(lapse), oa(a), ob(b)
    {
    }

    virtual ~MetricLFIntegrator()
    {
    }

    virtual void AssembleRHSElementVect(const FiniteElement &el, ElementTransformation &Tr, Vector &elvect)
    {
        const int vdim = 6;
        const int dof = el.GetDof();

        curvature_values.SetSize(vdim);

        shape.SetSize(dof);
        elvect.SetSize(dof * vdim);
        elvect = 0.0;

        const IntegrationRule *ir = IntRule;
        if (ir == NULL)
        {
            ir = &IntRules.Get(el.GetGeomType(), oa * el.GetOrder() + ob);
        }

        for (int i = 0; i < ir->GetNPoints(); i++)
        {
            const IntegrationPoint &ip = ir->IntPoint(i);
            Tr.SetIntPoint(&ip);

            double weight = Tr.Weight() * ip.weight;

            el.CalcPhysShape(Tr, shape);

            curvature.GetVectorValue(Tr, ip, curvature_values);
            lapse_value = lapse.GetValue(Tr, ip);

#define METRIC_RHS(i, j) -2.0 * lapse_value *curvature_values(##INDICES(i, j)##)

            double rhs[vdim] = {METRIC_RHS(1, 1),
                                METRIC_RHS(1, 2),
                                METRIC_RHS(1, 3),
                                METRIC_RHS(2, 2),
                                METRIC_RHS(2, 3),
                                METRIC_RHS(3, 3)};

            for (int k = 0; k < vdim; k++)
            {
                for (int s = 0; s < dof; s++)
                {
                    elvect(dof * k + s) += weight * rhs[k] * shape(s);
                }
            }
        }
    }
};

class CurvatureLFIntegrator : public LinearFormIntegrator
{
protected:
    GridFunction &metric;
    GridFunction &curvature;

    GridFunction &lapse;

    Vector metric_dof_values;
    Vector curvature_dof_values;
    Vector lapse_dof_values;

    Vector metric_values;
    DenseMatrix metric_gradients;
    DenseMatrix metric_hessians;
    Vector curvature_values;
    double lapse_value;
    Vector lapse_hessian;

    Vector shape;
    DenseMatrix dshape;
    DenseMatrix hshape;
    int oa, ob;

public:
    CurvatureLFIntegrator(GridFunction &metric, GridFunction &curvature, GridFunction &lapse, int a = 2, int b = 0)
        : metric(metric), curvature(curvature), lapse(lapse), oa(a), ob(b)
    {
    }

    virtual ~CurvatureLFIntegrator()
    {
    }

    virtual void AssembleRHSElementVect(const FiniteElement &el, ElementTransformation &Tr, Vector &elvect)
    {
        const int dim = 3;
        const int vdim = 6;
        const int dof = el.GetDof();

        metric_values.SetSize(vdim);
        metric_gradients.SetSize(vdim, dim);
        metric_hessians.SetSize(vdim, dim * (dim + 1) / 2);
        curvature_values.SetSize(vdim);
        lapse_hessian.SetSize(dim * (dim + 1) / 2);

        shape.SetSize(dof);
        dshape.SetSize(dof, dim);
        hshape.SetSize(dof, dim * (dim + 1) / 2);
        elvect.SetSize(dof * vdim);
        elvect = 0.0;

        const IntegrationRule *ir = IntRule;
        if (ir == NULL)
        {
            ir = &IntRules.Get(el.GetGeomType(), oa * el.GetOrder() + ob);
        }

        metric.GetElementDofValues(Tr.ElementNo, metric_dof_values);
        curvature.GetElementDofValues(Tr.ElementNo, curvature_dof_values);
        lapse.GetElementDofValues(Tr.ElementNo, lapse_dof_values);

        for (int i = 0; i < ir->GetNPoints(); i++)
        {
            const IntegrationPoint &ip = ir->IntPoint(i);
            Tr.SetIntPoint(&ip);

            double weight = Tr.Weight() * ip.weight;

            el.CalcPhysShape(Tr, shape);
            el.CalcPhysDShape(Tr, dshape);
            el.CalcPhysHessian(Tr, hshape);

            metric_values = 0.0;
            metric_gradients = 0.0;
            metric_hessians = 0.0;
            curvature_values = 0.0;
            lapse_value = 0.0;
            lapse_hessian = 0.0;

            for (int k = 0; k < vdim; k++)
            {
                for (int df = 0; df < dof; df++)
                {
                    metric_values(k) += shape(df) * metric_dof_values(k * dof + df);
                    curvature_values(k) += shape(df) * curvature_dof_values(k * dof + df);
                    for (int d = 0; d < dim; d++)
                    {
                        metric_gradients(k, d) += dshape(df, d) * metric_dof_values(k * dof + df);
                    }
                    for (int d = 0; d < dim * (dim + 1) / 2; d++)
                    {
                        metric_hessians(k, d) += hshape(df, d) * metric_dof_values(k * dof + df);
                    }
                }
            }

            for (int df = 0; df < dof; df++)
            {
                lapse_value += shape(df) * lapse_dof_values(df);
                for (int d = 0; d < dim * (dim + 1) / 2; d++)
                {
                    lapse_hessian(d) += hshape(df, d) * lapse_dof_values(df);
                }
            }

#define METRIC_VALUE(i, j) metric_values(##INDICES(i, j)##)
#define METRIC_GRAD(i, j, k) metric_gradients(##INDICES(i, j)##, k - 1)
#define METRIC_HESSIAN(i, j, k, l) metric_hessians(##INDICES(i, j)##, ##INDICES(k, l)##)

#define CURVATURE_VALUE(i, j) CONCAT(curvature_values[, CONCAT(INDICES(i, j),]))

            double metric_det = METRIC_VALUE(1, 1) * (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(3, 2)) -
                                METRIC_VALUE(1, 2) * (METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(1, 3)) +
                                METRIC_VALUE(1, 3) * (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(2, 2) * METRIC_VALUE(1, 3));

#define LAPSE_VALUE lapse_value
#define LAPSE_HESSIAN(k, l) lapse_hessian(##INDICES(k, l)##)

#define METRIC_DET_VALUE metric_det

            auto inv_metric_det = 1.0 / metric_det;

            auto inv_metric_values_0 = (METRIC_VALUE(2, 2) * METRIC_VALUE(3, 3) - METRIC_VALUE(2, 3) * METRIC_VALUE(2, 3)) * inv_metric_det;
            auto inv_metric_values_1 = (METRIC_VALUE(1, 3) * METRIC_VALUE(2, 3) - METRIC_VALUE(1, 2) * METRIC_VALUE(3, 3)) * inv_metric_det;
            auto inv_metric_values_2 = (METRIC_VALUE(1, 2) * METRIC_VALUE(2, 3) - METRIC_VALUE(1, 3) * METRIC_VALUE(2, 2)) * inv_metric_det;
            auto inv_metric_values_3 = (METRIC_VALUE(1, 1) * METRIC_VALUE(3, 3) - METRIC_VALUE(1, 3) * METRIC_VALUE(1, 3)) * inv_metric_det;
            auto inv_metric_values_4 = (METRIC_VALUE(1, 2) * METRIC_VALUE(1, 3) - METRIC_VALUE(1, 1) * METRIC_VALUE(2, 3)) * inv_metric_det;
            auto inv_metric_values_5 = (METRIC_VALUE(1, 1) * METRIC_VALUE(2, 2) - METRIC_VALUE(1, 2) * METRIC_VALUE(1, 2)) * inv_metric_det;

#define INV_METRIC_VALUE(i, j) CONCAT(inv_metric_values_, INDICES(i, j))

#define CONNECTION_TERM(i, j, k, l) 0.5 * INV_METRIC_VALUE(i, l) * (METRIC_GRAD(l, j, k) + METRIC_GRAD(l, k, j) - METRIC_GRAD(j, k, l))
#define COMPUTE_CONNECTION(i, j, k) CONNECTION_TERM(i, j, k, 1) + CONNECTION_TERM(i, j, k, 2) + CONNECTION_TERM(i, j, k, 3)

            auto connection_0_1 = COMPUTE_CONNECTION(1, 1, 1);
            auto connection_0_2 = COMPUTE_CONNECTION(1, 1, 2);
            auto connection_0_3 = COMPUTE_CONNECTION(1, 1, 3);

            auto connection_1_1 = COMPUTE_CONNECTION(1, 2, 1);
            auto connection_1_2 = COMPUTE_CONNECTION(1, 2, 2);
            auto connection_1_3 = COMPUTE_CONNECTION(1, 2, 3);

            auto connection_2_1 = COMPUTE_CONNECTION(1, 3, 1);
            auto connection_2_2 = COMPUTE_CONNECTION(1, 3, 2);
            auto connection_2_3 = COMPUTE_CONNECTION(1, 3, 3);

            auto connection_3_1 = COMPUTE_CONNECTION(2, 2, 1);
            auto connection_3_2 = COMPUTE_CONNECTION(2, 2, 2);
            auto connection_3_3 = COMPUTE_CONNECTION(2, 2, 3);

            auto connection_4_1 = COMPUTE_CONNECTION(2, 3, 1);
            auto connection_4_2 = COMPUTE_CONNECTION(2, 3, 2);
            auto connection_4_3 = COMPUTE_CONNECTION(2, 3, 3);

            auto connection_5_1 = COMPUTE_CONNECTION(3, 3, 1);
            auto connection_5_2 = COMPUTE_CONNECTION(3, 3, 2);
            auto connection_5_3 = COMPUTE_CONNECTION(3, 3, 3);

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

            auto ricci_0 = COMPUTE_RICCI(1, 1);
            auto ricci_1 = COMPUTE_RICCI(1, 2);
            auto ricci_2 = COMPUTE_RICCI(1, 3);
            auto ricci_3 = COMPUTE_RICCI(2, 2);
            auto ricci_4 = COMPUTE_RICCI(2, 3);
            auto ricci_5 = COMPUTE_RICCI(3, 3);

            // Define to access ricci
#define RICCI(i, j) CONCAT(ricci_, INDICES(i, j))

            // Compute curvature trace

            auto curvature_trace = INV_METRIC_VALUE(1, 1) * CURVATURE_VALUE(1, 1) + 2.0 * INV_METRIC_VALUE(1, 2) * CURVATURE_VALUE(1, 2) + 2.0 * INV_METRIC_VALUE(1, 3) * CURVATURE_VALUE(1, 3) +
                                   INV_METRIC_VALUE(2, 2) * CURVATURE_VALUE(2, 2) + 2.0 * INV_METRIC_VALUE(2, 3) * CURVATURE_VALUE(2, 3) +
                                   INV_METRIC_VALUE(3, 3) * CURVATURE_VALUE(3, 3);

#define CURVATURE_TRACE curvature_trace

#define CURVATURE_VALUE_BY_TRACE(i, j) CURVATURE_TRACE *CURVATURE_VALUE(i, j)
#define CURVATURE_INNER_PRODUCT_TERM(i, j, k, l) CURVATURE_VALUE(i, k) * INV_METRIC_VALUE(k, l) * CURVATURE_VALUE(l, j)
#define CURVATURE_INNER_PRODUCT(i, j) CURVATURE_INNER_PRODUCT_TERM(i, j, 1, 1) + CURVATURE_INNER_PRODUCT_TERM(i, j, 1, 2) + CURVATURE_INNER_PRODUCT_TERM(i, j, 1, 3) +     \
                                          CURVATURE_INNER_PRODUCT_TERM(i, j, 2, 1) + CURVATURE_INNER_PRODUCT_TERM(i, j, 2, 2) + CURVATURE_INNER_PRODUCT_TERM(i, j, 3, 3) + \
                                          CURVATURE_INNER_PRODUCT_TERM(i, j, 3, 1) + CURVATURE_INNER_PRODUCT_TERM(i, j, 3, 2) + CURVATURE_INNER_PRODUCT_TERM(i, j, 3, 3)

#define CURVATURE_RHS(i, j) LAPSE_VALUE *(RICCI(i, j) - 2.0 * CURVATURE_INNER_PRODUCT(i, j) + CURVATURE_VALUE_BY_TRACE(i, j)) - LAPSE_HESSIAN(i, j)

            double rhs[vdim] = {CURVATURE_RHS(1, 1),
                                CURVATURE_RHS(1, 2),
                                CURVATURE_RHS(1, 3),
                                CURVATURE_RHS(2, 2),
                                CURVATURE_RHS(2, 3),
                                CURVATURE_RHS(3, 3)};

            for (int k = 0; k < vdim; k++)
            {
                for (int s = 0; s < dof; s++)
                {
                    elvect(dof * k + s) += rhs[k] * weight * shape(s);
                }
            }
        }
    }
};

class EvolutionOperator : public TimeDependentOperator
{
protected:
    FiniteElementSpace *scalar_fespace;
    FiniteElementSpace *tensor_fespace;

    Array<int> scalar_ess_tdofs;
    Array<int> tensor_ess_tdofs;

    BilinearForm *tensor_mass_form;
    SparseMatrix tensor_mass_mat;

    BilinearForm *scalar_mass_form;
    SparseMatrix scalar_mass_mat;

    mutable GridFunction metric, curvature, lapse;

    DSmoother scalar_preconditioner; // Preconditioner for the mass matrix M
    DSmoother tensor_preconditioner;

    CGSolver scalar_solver; // Krylov solver for inverting the mass matrix M
    CGSolver tensor_solver;

public:
    EvolutionOperator(const Array<int> &ess_bdr, FiniteElementSpace *scalar_fespace, FiniteElementSpace *tensor_fespace)
        : TimeDependentOperator(2 * tensor_fespace->GetTrueVSize(), 0.0), scalar_fespace(scalar_fespace), tensor_fespace(tensor_fespace), metric(tensor_fespace), curvature(tensor_fespace), lapse(scalar_fespace)
    {
        std::cout << "Starting Constructor" << std::endl;

        scalar_fespace->GetEssentialTrueDofs(ess_bdr, scalar_ess_tdofs);
        tensor_fespace->GetEssentialTrueDofs(ess_bdr, tensor_ess_tdofs);

        const double rel_tol = 1e-8;

        std::cout << "Scalar" << std::endl;

        scalar_mass_form = new BilinearForm(scalar_fespace);
        scalar_mass_form->AddDomainIntegrator(new MassIntegrator());
        scalar_mass_form->Assemble();
        scalar_mass_form->FormSystemMatrix(scalar_ess_tdofs, scalar_mass_mat);

        std::cout << "Tensor" << std::endl;

        tensor_mass_form = new BilinearForm(tensor_fespace);

        std::cout << "Add" << std::endl;

        tensor_mass_form->AddDomainIntegrator(new VectorFEMassIntegrator());
        std::cout << "Addemble" << std::endl;
        tensor_mass_form->Assemble();
        std::cout << "Form" << std::endl;
        tensor_mass_form->FormSystemMatrix(tensor_ess_tdofs, tensor_mass_mat);

        std::cout << "Scalar Solver" << std::endl;

        scalar_solver.iterative_mode = false;
        scalar_solver.SetRelTol(rel_tol);
        scalar_solver.SetAbsTol(0.0);
        scalar_solver.SetMaxIter(30);
        scalar_solver.SetPrintLevel(0);
        scalar_solver.SetPreconditioner(scalar_preconditioner);
        scalar_solver.SetOperator(scalar_mass_mat);

        std::cout << "Tensor Solver" << std::endl;

        tensor_solver.iterative_mode = false;
        tensor_solver.SetRelTol(rel_tol);
        tensor_solver.SetAbsTol(0.0);
        tensor_solver.SetMaxIter(30);
        tensor_solver.SetPrintLevel(0);
        tensor_solver.SetPreconditioner(tensor_preconditioner);
        tensor_solver.SetOperator(tensor_mass_mat);
    }

    virtual ~EvolutionOperator()
    {
        delete scalar_mass_form;
        delete tensor_mass_form;
    }

    /// Compute the right-hand side of the ODE system.
    virtual void
    Mult(const Vector &spacetime, Vector &dt_spacetime) const
    {
        Vector metric_vec(spacetime.GetData() + 0, tensor_fespace->GetTrueVSize());
        Vector curvature_vec(spacetime.GetData() + tensor_fespace->GetTrueVSize(), tensor_fespace->GetTrueVSize());

        Vector dt_metric_vec(dt_spacetime.GetData() + 0, tensor_fespace->GetTrueVSize());
        Vector dt_curvature_vec(dt_spacetime.GetData() + tensor_fespace->GetTrueVSize(), tensor_fespace->GetTrueVSize());

        metric.SetFromTrueDofs(metric_vec);
        curvature.SetFromTrueDofs(curvature_vec);

        LinearForm lapse_rhs(scalar_fespace);

        lapse_rhs.AddDomainIntegrator(new LapseLFIntegrator(metric));
        lapse_rhs.Assemble();

        // Solve
        scalar_solver.Mult(lapse_rhs, lapse);

        LinearForm metric_rhs(tensor_fespace);

        metric_rhs.AddDomainIntegrator(new MetricLFIntegrator(curvature, lapse));
        metric_rhs.Assemble();

        LinearForm curvature_rhs(tensor_fespace);
        curvature_rhs.AddDomainIntegrator(new CurvatureLFIntegrator(metric, curvature, lapse));
        curvature_rhs.Assemble();

        tensor_solver.Mult(metric_rhs, dt_metric_vec);
        tensor_solver.Mult(curvature_rhs, dt_curvature_vec);
    }
};

struct Particle
{
    double x, y, z;
    double velx, vely, velz;
    double mass;
};

struct ParticleSolverDescriptor
{
    Constants constants;

    int particle_count;
    Particle *particles;

    int element_order;

    double domain_width;
    double domain_height;
    double domain_depth;
    int domain_refinement;
};

struct ParticleSolver
{
    Constants constants;
    std::vector<Particle> particles;

    Mesh *mesh;
    FiniteElementCollection *fec;
    // FiniteElementSpace *fespace;

    FiniteElementSpace *scalar_fe_space;
    FiniteElementSpace *tensor_fe_space;
    Array<int> tensor_boundry_dofs;

    BlockVector *spacetime;

    EvolutionOperator *evolution;
    ODESolver *ode_solver;
};

void initial_metric(const Vector &point, Vector &metric)
{
    metric = 0.0;
    metric[0] = 1.0;
    metric[3] = 1.0;
    metric[5] = 1.0;
}

void initial_curvature(const Vector &point, Vector &curvature)
{
    curvature = 0.0;
}

ENGINE_API void *particle_solver_create(ParticleSolverDescriptor desc)
{
    ParticleSolver *solver = new ParticleSolver();
    solver->constants = desc.constants;

    const int dim = 3;

    ////////////////////////
    // Create Mesh /////////
    ////////////////////////

    solver->mesh = new Mesh(dim, 8, 1);

    double hex_v[8][3] =
        {
            {-1, -1, -1}, {+1, -1, -1}, {+1, +1, -1}, {-1, +1, -1}, {-1, -1, +1}, {+1, -1, +1}, {+1, +1, +1}, {-1, +1, +1}};

    for (int i = 0; i < 8; i++)
    {
        hex_v[i][0] *= desc.domain_width;
        hex_v[i][1] *= desc.domain_height;
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

    //////////////////////////
    // FE Collection /////////
    //////////////////////////

    solver->fec = new H1_FECollection(desc.element_order, dim);

    /////////////////////////
    // FE Space /////////////
    /////////////////////////

    solver->scalar_fe_space = new FiniteElementSpace(solver->mesh, solver->fec);
    solver->tensor_fe_space = new FiniteElementSpace(solver->mesh, solver->fec, 6, Ordering::byVDIM);

    int tdofs = solver->tensor_fe_space->GetTrueVSize();

    std::cout
        << "Creating solver with unknowns: " << tdofs << std::endl;

    /////////////////////////
    // Spacetime Quantities /
    /////////////////////////

    std::cout << "Spacetime Quantities" << std::endl;

    Array<int> offsets(3);
    offsets[0] = 0;
    offsets[1] = tdofs;
    offsets[2] = 2 * tdofs; // total size

    solver->spacetime = new BlockVector(offsets);

    GridFunction metric, curvature;
    metric.MakeTRef(solver->tensor_fe_space, solver->spacetime->GetBlock(0), 0);
    curvature.MakeTRef(solver->tensor_fe_space, solver->spacetime->GetBlock(1), 0);

    metric.ProjectCoefficient(VectorFunctionCoefficient(6, initial_metric));
    metric.SetTrueVector();
    curvature.ProjectCoefficient(VectorFunctionCoefficient(6, initial_curvature));
    curvature.SetTrueVector();

    /////////////////////////
    // Particles ////////////
    /////////////////////////

    std::cout << "Particles" << std::endl;

    solver->particles = std::vector<Particle>(desc.particles, desc.particles + desc.particle_count);

    /////////////////////////
    // Solver ///////////////
    /////////////////////////

    std::cout << "Solver" << std::endl;

    solver->ode_solver = new RK2Solver(0.5);

    std::cout << "Evolution" << std::endl;

    Array<int> ess_bdr(solver->mesh->bdr_attributes.Max());
    ess_bdr = 0;
    ess_bdr[0] = 1; // boundary attribute 1 (index 0) is fixed

    solver->evolution = new EvolutionOperator(ess_bdr, solver->scalar_fe_space, solver->tensor_fe_space);

    double t = 0.0;
    double t_final = 0.5;
    double dt = 1.0e-2;

    solver->evolution->SetTime(t);

    std::cout << "Inniting" << std::endl;

    solver->ode_solver->Init(*solver->evolution);

    std::cout << "ODE Solver Running" << std::endl;

    bool last_step = false;
    for (int ti = 1; !last_step; ti++)
    {
        std::cout << "Step" << std::endl;

        if (t + dt >= t_final - dt / 2)
        {
            last_step = true;
        }

        solver->ode_solver->Step(*solver->spacetime, t, dt);
    }

    return solver;
}

ENGINE_API void particle_solver_update(void *p_solver, double t, double delta)
{
    ParticleSolver *solver = (ParticleSolver *)p_solver;
}

ENGINE_API void particle_solver_destroy(void *p_solver)
{
    ParticleSolver *solver = (ParticleSolver *)p_solver;

    delete solver->ode_solver;
    delete solver->evolution;

    delete solver->spacetime;

    delete solver->tensor_fe_space;
    delete solver->fec;
    delete solver->mesh;

    delete solver;
}

ENGINE_API Particle particle_solver_get_particle(void *p_solver, unsigned int index)
{
    ParticleSolver *solver = (ParticleSolver *)p_solver;

    return solver->particles[index];
}

int main()
{
    try
    {
        ParticleSolverDescriptor desc = ParticleSolverDescriptor{
            Constants{1.0, 1.0}, 0, nullptr, 2, 1.0, 1.0, 1.0, 2};

        void *solver = particle_solver_create(desc);
        particle_solver_destroy(solver);
    }
    catch (const std::exception &e)
    {
        std::cerr << e.what() << std::endl;
    }
}
