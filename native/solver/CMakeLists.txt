cmake_minimum_required(VERSION 3.22)

add_library(solver SHARED)
DEAL_II_SETUP_TARGET(solver)

set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

target_include_directories(solver PRIVATE ${SRC_DIR})

target_sources(solver PRIVATE ${SRC_DIR}/base/nbody.h ${SRC_DIR}/base/defines.h 
                                        ${SRC_DIR}/context/context.hpp ${SRC_DIR}/context/context.cpp 
                                        ${SRC_DIR}/context/mesh.hpp ${SRC_DIR}/context/mesh.cpp 
                                        ${SRC_DIR}/source/nbody.hpp ${SRC_DIR}/source/nbody.cpp
                                        ${SRC_DIR}/solver/post_newtonian.cpp)

set_target_properties(solver PROPERTIES PREFIX "")
set_target_properties(solver PROPERTIES SUFFIX ".cdylib")

install(TARGETS solver DESTINATION $ENV{CDYLIB_DIR})