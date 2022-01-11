add_library(spacetime SHARED)

# target_link_libraries(spacetime PRIVATE dealii)

set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

target_include_directories(spacetime PRIVATE ${SRC_DIR})

target_sources(spacetime PRIVATE ${SRC_DIR}/entry.cpp ${SRC_DIR}/entry.hpp)

set_target_properties(spacetime PROPERTIES SUFFIX .cdylib)

install(TARGETS spacetime DESTINATION ${SHARED_LIB_INSTALL_DIR})