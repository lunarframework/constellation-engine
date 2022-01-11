cmake_minimum_required(VERSION 3.15)

###############################
## Third Party ################
###############################

# Add dealii
# add_subdirectory(third-party/dealii)

###############################
## Project ####################
###############################

# Set name
project(native LANGUAGES CXX VERSION 1.0)

# Specify the C++ 17 standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(SHARED_LIB_INSTALL_DIR $ENV{CDYLIB_DIR})

add_subdirectory(spacetime)