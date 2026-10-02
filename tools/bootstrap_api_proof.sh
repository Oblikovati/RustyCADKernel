#!/usr/bin/env bash
# Provision local API inventory tools and configure the pinned, all-module OCCT tree.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
config="$repo_root/porting/api-proof-toolchain.json"
cache_root="${XDG_CACHE_HOME:-$HOME/.cache}/rustycad-api-proof"
llvm_root="$cache_root/llvm-23.1.2"
llvm_archive="$cache_root/LLVM-23.1.2-Linux-X64.tar.zst"
occt_source="${OCCT_SOURCE:-$repo_root/../OCCT}"
build_root="$repo_root/target/api-proof/occt-build"
compile_db="$build_root/compile_commands.json"

read_config() {
  python3 - "$config" "$1" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as config_file:
    config = json.load(config_file)
value = config
for part in sys.argv[2].split("."):
    value = value[part]
if isinstance(value, list):
    print("\n".join(value))
else:
    print(value)
PY
}

check_host() {
  [[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] || {
    echo "Pinned LLVM archive supports Linux x86_64; host is $(uname -s) $(uname -m)." >&2
    return 1
  }
  command -v curl >/dev/null && command -v sha256sum >/dev/null && command -v tar >/dev/null
  command -v zstd >/dev/null
  command -v cmake >/dev/null && command -v python3 >/dev/null
}

download_llvm() {
  [[ -f "$llvm_archive" ]] && return
  echo "Downloading pinned LLVM $(read_config llvm_version) ($(read_config llvm_archive)); archive is about 1.2 GB." >&2
  curl --fail --location --proto '=https' --tlsv1.2 "$(read_config llvm_url)" --output "$llvm_archive"
}

verify_llvm_archive() {
  local sha
  sha="$(read_config llvm_sha256)"
  echo "$sha  $llvm_archive" | sha256sum --check --status || {
    rm -f "$llvm_archive"
    echo "LLVM archive SHA-256 does not match the checked-in pin." >&2
    return 1
  }
}

extract_llvm() {
  local version unpack_root
  version="$(read_config llvm_version)"
  unpack_root="$cache_root/unpack-llvm-$$"
  mkdir -p "$unpack_root"
  if ! zstd --long=30 -dc "$llvm_archive" | tar -xf - -C "$unpack_root"; then
    rm -rf "$unpack_root"
    return 1
  fi
  [[ -x "$unpack_root/LLVM-$version-Linux-X64/bin/clang++" ]] || { rm -rf "$unpack_root"; return 1; }
  mv "$unpack_root/LLVM-$version-Linux-X64" "$llvm_root"
  rmdir "$unpack_root"
}

install_llvm() {
  mkdir -p "$cache_root"
  if [[ ! -x "$llvm_root/bin/clang++" ]]; then
    download_llvm
    verify_llvm_archive
    extract_llvm
  fi
  "$llvm_root/bin/clang++" --version | head -n 1
}

install_rustdoc_nightly() {
  export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
  if ! command -v rustup >/dev/null; then
    echo "rustup is required; install it with tools/bootstrap.sh first." >&2
    return 1
  fi
  rustup toolchain install "$(read_config rust_nightly)" --profile minimal
  rustc +"$(read_config rust_nightly)" --version
}

check_occt_revision() {
  local expected actual
  expected="$(read_config occt_revision)"
  actual="$(git -C "$occt_source" rev-parse HEAD)"
  [[ "$actual" == "$expected" ]] || { echo "Expected OCCT $expected, got $actual." >&2; return 1; }
}

cxx_standard_flags() {
  local cxx_headers cxx_version cxx_arch_headers
  cxx_headers="$(find /usr/include/c++ -mindepth 2 -maxdepth 2 -name algorithm -print -quit)"
  [[ -n "$cxx_headers" ]] || {
    echo "C++ standard library headers are needed for Clang parsing; none found under /usr/include/c++." >&2
    return 2
  }
  cxx_version="$(basename "$(dirname "$cxx_headers")")"
  printf '%s' "-isystem $(dirname "$cxx_headers")"
  cxx_arch_headers="/usr/include/x86_64-linux-gnu/c++/$cxx_version"
  [[ ! -d "$cxx_arch_headers" ]] || printf ' -isystem %s' "$cxx_arch_headers"
  [[ ! -d "$(dirname "$cxx_headers")/backward" ]] || printf ' -isystem %s/backward' "$(dirname "$cxx_headers")"
}

configure_occt() {
  check_occt_revision
  [[ -x "$llvm_root/bin/clang++" ]] || { echo "Run '$0 install' before configuring OCCT." >&2; return 1; }
  local cxx_flags module
  local -a module_args=()
  cxx_flags="$(cxx_standard_flags)"
  while IFS= read -r module; do module_args+=("-DBUILD_MODULE_${module}=ON"); done < <(read_config modules)
  cmake -S "$occt_source" -B "$build_root" \
    -DCMAKE_C_COMPILER="$llvm_root/bin/clang" \
    -DCMAKE_CXX_COMPILER="$llvm_root/bin/clang++" \
    -DCMAKE_CXX_FLAGS="$cxx_flags" \
    -DCMAKE_EXPORT_COMPILE_COMMANDS=ON \
    -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY \
    -DBUILD_DOC_Overview=OFF -DBUILD_DOC_RefMan=OFF \
    -DUSE_TBB=OFF "${module_args[@]}"
  python3 "$repo_root/tools/api_proof_config.py" record "$config" \
    "$repo_root/porting/inventory.json" "$occt_source" "$build_root"
}

verify() {
  verify_llvm_archive
  "$llvm_root/bin/clang++" --version | head -n 1
  export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
  rustc +"$(read_config rust_nightly)" --version
  python3 "$repo_root/tools/api_proof_config.py" verify "$compile_db" \
    "$build_root/api-proof-config.json" "$config" "$occt_source"
}

usage() {
  echo "Usage: $0 {install|configure|all|verify}" >&2
  echo "Set OCCT_SOURCE to use another checkout of the pinned OCCT revision." >&2
}

check_host
case "${1:-}" in
  install) install_llvm; install_rustdoc_nightly ;;
  configure) configure_occt ;;
  all) install_llvm; install_rustdoc_nightly; configure_occt ;;
  verify) verify ;;
  *) usage; exit 2 ;;
esac
