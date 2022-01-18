#pragma once

// Macro magic to allow expanding macros in macros
#define CONCAT1(first, second) first##second
#define CONCAT(first, second) CONCAT1(first, second)

#define SOLVER_API extern "C" __declspec(dllexport)