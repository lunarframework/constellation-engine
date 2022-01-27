#include "base/defines.h"
#include "context/context.hpp"
#include "context/mesh.hpp"
#include "source/nbody.hpp"

#include <unordered_map>
#include <utility>
#include <cmath>
#include <iostream>

struct NBodySourcePair
{
    NBodySource *p_source;
    NBodySourceData data;
};

struct PostNewtonianSolver
{
    Context *p_context;

    std::unordered_map<NBodySource *, NBodySourceData> n_body_sources;
};

SOLVER_API PostNewtonianSolver *post_newtonian_solver_create(Context *p_context)
{
    PostNewtonianSolver *p_solver = new PostNewtonianSolver();
    p_solver->p_context = p_context;
    p_solver->n_body_sources = std::unordered_map<NBodySource *, NBodySourceData>();

    return p_solver;
}

SOLVER_API void post_newtonian_solver_attach_n_body_source(PostNewtonianSolver *p_solver, NBodySource *p_source)
{
    p_solver->n_body_sources.emplace(std::make_pair(p_source, NBodySourceData{}));
}

SOLVER_API void post_newtonian_solver_detach_n_body_source(PostNewtonianSolver *p_solver, NBodySource *p_source)
{
    p_solver->n_body_sources.erase(p_solver->n_body_sources.find(p_source));
}

SOLVER_API NBodySourceData *post_newtonian_solver_n_body_source_data(PostNewtonianSolver *p_solver, NBodySource *p_source)
{
    return &p_solver->n_body_sources.at(p_source);
}

SOLVER_API void post_newtonian_solver_run(PostNewtonianSolver *p_solver, double delta, unsigned int steps)
{
    for (auto &source : p_solver->n_body_sources)
    {
        auto n = source.first->nbodies.size();
        source.second.n = n;
        source.second.max_time = delta * steps;
        source.second.steps = steps;
        source.second.nbodies = std::vector<NBody>();
        source.second.nbodies.reserve((steps + 1) * n);
    }

    auto time = 0;

    auto g = p_solver->p_context->gravitational_constant;
    auto c = p_solver->p_context->speed_of_light;
    auto c_sq = c * c;

    for (unsigned int i = 0; i < steps; i++)
    {
        for (auto &source : p_solver->n_body_sources)
        {
            source.second.nbodies.insert(source.second.nbodies.end(), source.first->nbodies.begin(), source.first->nbodies.end());
        }

        for (auto &source1 : p_solver->n_body_sources)
        {
            for (unsigned int orbit_id = 0; orbit_id < source1.first->nbodies.size(); orbit_id++)
            {
                auto &orbit = source1.first->nbodies[orbit_id];

                double accx = 0.0;
                double accy = 0.0;
                double accz = 0.0;

                for (auto &source2 : p_solver->n_body_sources)
                {

                    for (unsigned int grav_id = 0; grav_id < source2.first->nbodies.size(); grav_id++)
                    {
                        // if (source1.first != source2.first || grav_id != orbit_id) {

                        // }

                        auto &grav = source2.first->nbodies[grav_id];

                        double x = orbit.x - grav.x;
                        double y = orbit.y - grav.y;
                        double z = orbit.z - grav.z;

                        double r = sqrt(x * x + y * y + z * z);

                        if (r < 1e-10)
                        {
                            continue;
                        }

                        double velx = orbit.velx - grav.velx;
                        double vely = orbit.vely - grav.vely;
                        double velz = orbit.velz - grav.velz;

                        double mu = g * grav.mass;

                        double force_over_r = -mu / (r * r * r);

                        double m = (mu) / (2.0 * c_sq * r);
                        double one_over_c_sq_one_plus_m = 1.0 / (c_sq * (1.0 + m));
                        double one_minus_m_over_one_plus_m = (1.0 - m) / ((1.0 + m) * (1.0 + m) * (1.0 + m) * (1.0 + m) * (1.0 + m) * (1.0 + m) * (1.0 + m));

                        double velx_sq = velx * velx;
                        double vely_sq = vely * vely;
                        double velz_sq = velz * velz;
                        double xvelx = x * velx;
                        double yvely = y * vely;
                        double zvelz = z * velz;

                        double pos_dot_vel_over_one_minus_m = (xvelx + yvely + zvelz) / (1.0 - m);

                        accx += force_over_r * (one_minus_m_over_one_plus_m * x - one_over_c_sq_one_plus_m * (x * (velx_sq - vely_sq - velz_sq) +
                                                                                                              2.0 * velx * (yvely + zvelz + pos_dot_vel_over_one_minus_m)));

                        accy += force_over_r * (one_minus_m_over_one_plus_m * y - one_over_c_sq_one_plus_m * (y * (vely_sq - velx_sq - velz_sq) +
                                                                                                              2.0 * vely * (xvelx + zvelz + pos_dot_vel_over_one_minus_m)));

                        accz += force_over_r * (one_minus_m_over_one_plus_m * z - one_over_c_sq_one_plus_m * (z * (velz_sq - vely_sq - velx_sq) +
                                                                                                              2.0 * velz * (yvely + xvelx + pos_dot_vel_over_one_minus_m)));
                    }
                }

                // if (i == 0)
                // {
                //     orbit.velx += accx * delta;
                //     orbit.vely += accy * delta;
                //     orbit.velz += accz * delta;

                //     orbit.x += orbit.velx * delta;
                //     orbit.y += orbit.vely * delta;
                //     orbit.z += orbit.velz * delta;
                // }
                // else
                // {
                // }

                orbit.velx += accx * delta;
                orbit.vely += accy * delta;
                orbit.velz += accz * delta;

                orbit.x += orbit.velx * delta;
                orbit.y += orbit.vely * delta;
                orbit.z += orbit.velz * delta;
            }
        }

        time += delta;
    }

    for (auto &source : p_solver->n_body_sources)
    {
        source.second.nbodies.insert(source.second.nbodies.end(), source.first->nbodies.begin(), source.first->nbodies.end());
    }
}

SOLVER_API void post_newtonian_solver_destroy(PostNewtonianSolver *p_solver)
{
    delete p_solver;
}