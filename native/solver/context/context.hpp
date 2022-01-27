#pragma once

#include "base/defines.h"

struct Context
{
    double speed_of_light;
    double gravitational_constant;
};

struct ContextDescriptor
{
    double speed_of_light;
    double gravitational_constant;
};

SOLVER_API Context *context_create(ContextDescriptor descriptor);

SOLVER_API void context_destroy(Context *p_context);
