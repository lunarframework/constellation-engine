#include <source/nbody.hpp>

SOLVER_API NBodySource *n_body_source_create()
{
    return new NBodySource{std::vector<NBody>{}};
}

SOLVER_API void n_body_source_add(NBodySource *p_source, NBody n_body)
{
    p_source->nbodies.push_back(n_body);
}

SOLVER_API void n_body_source_destroy(NBodySource *p_source)
{
    delete p_source;
}

SOLVER_API double n_body_source_data_max_time(NBodySourceData *p_data)
{
    return p_data->max_time;
}

SOLVER_API unsigned int n_body_source_data_steps(NBodySourceData *p_data)
{
    return p_data->steps;
}

SOLVER_API unsigned int n_body_source_data_n(NBodySourceData *p_data)
{
    return p_data->n;
}

SOLVER_API NBody *n_body_source_data_slice(NBodySourceData *p_data, unsigned int i)
{
    return &(p_data->nbodies.at(i * p_data->n));
}

SOLVER_API void n_body_source_data_destroy(NBodySourceData *p_data)
{
    delete p_data;
}