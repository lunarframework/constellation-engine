#include "context/context.hpp"

SOLVER_API Context *context_create(ContextDescriptor descriptor)
{
    Context *p_context = new Context();
    p_context->speed_of_light = descriptor.speed_of_light;
    p_context->gravitational_constant = descriptor.gravitational_constant;

    return p_context;
}

SOLVER_API void context_destroy(Context *p_context)
{
    delete p_context;
}