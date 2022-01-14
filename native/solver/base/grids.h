#pragma once

#include "stdint.h"

typedef struct CubeGrid
{
    double width;
    double height;
    double depth;

    uint32_t refinement;
} CubeGrid;