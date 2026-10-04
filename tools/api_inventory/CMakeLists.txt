cmake_minimum_required(VERSION 3.20)
project(occt_api_extract LANGUAGES CXX)

find_package(LLVM 23.1.2 CONFIG REQUIRED)
find_package(Clang 23.1.2 CONFIG REQUIRED)
add_executable(occt-api-extract
  src/AliasReachability.cpp
  src/main.cpp
  src/InventoryVisitor.cpp
  src/PreprocessorEvents.cpp
  src/Schema.cpp)
target_include_directories(occt-api-extract PRIVATE include ${LLVM_INCLUDE_DIRS} ${CLANG_INCLUDE_DIRS})
target_compile_features(occt-api-extract PRIVATE cxx_std_17)
target_compile_options(occt-api-extract PRIVATE -fno-rtti)
target_compile_definitions(occt-api-extract PRIVATE ${LLVM_DEFINITIONS})
target_compile_definitions(occt-api-extract PRIVATE
  CLANG_DRIVER_PATH="${LLVM_TOOLS_BINARY_DIR}/clang++"
  CLANG_RESOURCE_DIR="${LLVM_LIBRARY_DIRS}/clang/23")
target_link_libraries(occt-api-extract PRIVATE clang-cpp)

include(CTest)
find_package(Python3 COMPONENTS Interpreter REQUIRED)
if(BUILD_TESTING)
  add_test(NAME api_inventory_fixture
    COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/check_fixture.py
      $<TARGET_FILE:occt-api-extract> ${CMAKE_CURRENT_SOURCE_DIR}/fixtures)
  add_test(NAME api_inventory_signature_identity
    COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/tests/check_merge_identity.py)
endif()
