cmake_minimum_required(VERSION 3.22)

###############################
## Project ####################
###############################

# Set name
project(engine-sys LANGUAGES C CXX)

# Specify the C++ 17 standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Packages
find_package(mfem 4.3 REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR}/packages/mfem)

message(STATUS "Found mfem config in ${mfem_DIR} (version ${MFEM_VERSION})")
message(STATUS "Mfem compiler is ${MFEM_CXX_COMPILER}")

if (NOT CMAKE_CXX_COMPILER AND MFEM_CXX_COMPILER)
    set(CMAKE_CXX_COMPILER ${MFEM_CXX_COMPILER})
endif()

# include_directories(${MFEM_INCLUDE_DIRS})

add_library(engine-sys STATIC)

target_include_directories(engine-sys PRIVATE ${MFEM_INCLUDE_DIRS})
target_link_libraries(engine-sys PRIVATE ${MFEM_LIBRARIES})

set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)

target_include_directories(engine-sys PRIVATE ${SRC_DIR})

target_sources(engine-sys PRIVATE ${SRC_DIR}/constants.hpp
                                    ${SRC_DIR}/export.hpp
                                    ${SRC_DIR}/generic/particle.cpp)
