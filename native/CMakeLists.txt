cmake_minimum_required(VERSION 3.22)

###############################
## Third Party ################
###############################

# FIND_PACKAGE(deal.II 9.3.2 REQUIRED
#   PATHS ${CMAKE_CURRENT_SOURCE_DIR}/third-party/dealii
#   )

# find_package(dealii 9.3.2 REQUIRED)

# add_subdirectory(third-party/dealii)

# DEAL_II_INITIALIZE_CACHED_VARIABLES()

###############################
## Project ####################
###############################

# Set name
project(native LANGUAGES CXX)

# Add dealii
find_package(deal.II 9.3.2 REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR}/packages/dealii)
DEAL_II_INITIALIZE_CACHED_VARIABLES()

# Specify the C++ 17 standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# set(SHARED_LIB_INSTALL_DIR $ENV{CONSTELLATION_TARGET_DIR})

add_subdirectory(solver)