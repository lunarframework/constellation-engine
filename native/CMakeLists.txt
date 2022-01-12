cmake_minimum_required(VERSION 3.22)

###############################
## Third Party ################
###############################

find_package(dealii 9.3.2 REQUIRED PATHS ${CONSTELLATION_DEALII_OUTPUT_DIR} NO_DEFAULT_PATH)

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
# execute_process(COMMAND ${CMAKE_COMMAND} ${CMAKE_CURRENT_SOURCE_DIR}/third-party/build
#     WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/third-party/dealii)

# Specify the C++ 17 standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# set(SHARED_LIB_INSTALL_DIR $ENV{CONSTELLATION_TARGET_DIR})

# add_subdirectory(spacetime)