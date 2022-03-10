cmake_minimum_required(VERSION 3.22)

###############################
## Project ####################
###############################

# Set name
project(engine-cpp LANGUAGES C CXX)

find_package(deal.II 9.3.2 REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR}/packages/dealii)
DEAL_II_INITIALIZE_CACHED_VARIABLES()

# Specify the C++ 17 standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Packages

# find_package(mfem 4.3 REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR}/packages/mfem)

# message(STATUS "Found mfem config in ${mfem_DIR} (version ${MFEM_VERSION})")
# message(STATUS "Mfem compiler is ${MFEM_CXX_COMPILER}")

# if (NOT CMAKE_CXX_COMPILER AND MFEM_CXX_COMPILER)
#     set(CMAKE_CXX_COMPILER ${MFEM_CXX_COMPILER})
# endif()

# # include_directories(${MFEM_INCLUDE_DIRS})

# add_library(engine-cpp SHARED)
add_executable(engine-cpp)
DEAL_II_SETUP_TARGET(engine-cpp)

# target_include_directories(engine-cpp PRIVATE ${MFEM_INCLUDE_DIRS})
# target_link_libraries(engine-cpp PRIVATE ${MFEM_LIBRARIES})

set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)

target_include_directories(engine-cpp PRIVATE ${SRC_DIR})

target_sources(engine-cpp PRIVATE ${SRC_DIR}/constants.hpp
                                    ${SRC_DIR}/export.hpp
                                    ${SRC_DIR}/generic/particle.cpp)

# foreach(LIBRARY ${MFEM_LIBRARIES})
#     install(TARGETS ${LIBRARY} DESTINATION ${CMAKE_CURRENT_SOURCE_DIR}/lib)
#     message("Installing " ${LIBRARY})
#  endforeach()

# Installing

# foreach(LIBRARY ${MFEM_LIBRARIES})
#     install(FILES $<TARGET_FILE:${LIBRARY}> DESTINATION ${CMAKE_CURRENT_SOURCE_DIR}/lib)
#     message("Installing " ${LIBRARY})
# endforeach()

install(TARGETS engine-cpp ARCHIVE DESTINATION ${CMAKE_CURRENT_SOURCE_DIR}/lib)