cmake_minimum_required(VERSION 3.22)

add_library(solver SHARED)
DEAL_II_SETUP_TARGET(solver)

set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

target_include_directories(solver PRIVATE ${SRC_DIR})

target_sources(solver PRIVATE ${SRC_DIR}/base/grids.h
        ${SRC_DIR}/generic_3d/solver_types.h ${SRC_DIR}/generic_3d/solver.cpp)

set_target_properties(solver PROPERTIES PREFIX "")
set_target_properties(solver PROPERTIES SUFFIX ".cdylib")

install(TARGETS solver DESTINATION $ENV{CDYLIB_DIR})