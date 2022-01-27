#pragma once

#include "base/defines.h"

#include <vector>

struct NBody
{
    double x, y, z;
    double velx, vely, velz;
    double mass;
};

struct NBodySource
{
    std::vector<NBody> nbodies;
};

SOLVER_API NBodySource *n_body_source_create();

SOLVER_API void n_body_source_add(NBodySource *p_source, NBody n_body);

SOLVER_API void n_body_source_destroy(NBodySource *p_source);

struct NBodySourceData
{
    unsigned int n;
    unsigned int steps;

    double max_time;

    std::vector<NBody> nbodies;
};

SOLVER_API double n_body_source_data_max_time(NBodySourceData *p_data);

SOLVER_API unsigned int n_body_source_data_steps(NBodySourceData *p_data);

SOLVER_API unsigned int n_body_source_data_n(NBodySourceData *p_data);

SOLVER_API NBody *n_body_source_data_slice(NBodySourceData *p_data, unsigned int i);