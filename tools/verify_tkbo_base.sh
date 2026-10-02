#!/usr/bin/env bash
# Compare base-layer behavior with the pinned native build and Rust regressions.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
occt_build="${OCCT_BUILD:?Set OCCT_BUILD to the pinned OCCT modeling build}"
occt_lib="${OCCT_LIB_DIR:-$occt_build/lin64/gcc/libi}"
cd "$repo_root"
mkdir -p target/tkbo-base-oracle
for suite in options progress; do
  "${CXX:-c++}" -std=c++17 -I"$occt_build/include/opencascade" \
    "tools/oracle/tkbo_${suite}.cxx" -L"$occt_lib" -Wl,-rpath,"$occt_lib" \
    -lTKBO -lTKernel -o "target/tkbo-base-oracle/$suite"
  "target/tkbo-base-oracle/$suite"
done
for suite in brep_algo_base brep_algo_builder; do
  "${CXX:-c++}" -std=c++17 -I"$occt_build/include/opencascade" \
    "tools/oracle/tkbo_${suite}.cxx" -L"$occt_lib" -Wl,-rpath,"$occt_lib" \
    -lTKBO -lTKTopAlgo -lTKBRep -lTKernel -o "target/tkbo-base-oracle/$suite"
  "target/tkbo-base-oracle/$suite"
done
cargo test --locked -p rustycad-tkbo --test bop_algo_options --test bop_algo_progress \
  --test bop_algo_enums --test brep_algo_api_algo --test brep_algo_api_builder_algo
