#!/usr/bin/env bash
# Builds Intel's GPU compute stack from source into PREFIX, for the release's Intel workers:
#
#   build-intel-gpu-runtime.sh <prefix>
#
# - the Level Zero loader (libze_loader.so.1), which the SYCL runtime opens;
# - the compute runtime "NEO" (libze_intel_gpu.so.1, the Level Zero driver; and ocloc, the
#   offline compiler: the build compiles kernels ahead of time with it, and the SYCL runtime loads
#   libocloc.so to check those kernels fit the card);
# - the graphics compiler IGC (libigc.so.2, libigdfcl.so.2, libopencl-clang2.so.16), which compiles
#   SPIR-V kernels for a card nothing was compiled for in advance;
# - the memory manager gmmlib (libigdgmm.so.12).
#
# Built here because Intel's own packages of these versions need glibc 2.38 (Ubuntu 24.04), and the
# release runs on glibc 2.35 (Ubuntu 22.04). The versions are the ones zkminer's Intel workers were
# validated with on an Arc B580: compute runtime 26.05.37020.3 with IGC 2.28.4 (the pair Intel
# released together) and Level Zero 1.32.0. Each source is pinned to a commit, checked after the
# fetch. IGC's dependencies are the revisions its v2.28.4 release notes list.
#
# Every library is given RUNPATH $ORIGIN, so the set finds itself wherever it is copied: the loader
# opens the driver, and the driver opens IGC, from the same directory, ahead of any system copy.
# Licenses go to PREFIX/share/licenses/<project>/.
set -euo pipefail

prefix=${1:?usage: $0 <prefix>}
src=${SRC_DIR:-/tmp/intel-gpu-src}
jobs=${JOBS:-$(nproc)}
mkdir -p "$src" "$prefix"

# <dir> <repository> <commit> [<tag or branch to fetch shallowly>]
fetch() {
  local dir=$1 repo=$2 commit=$3 ref=${4:-}
  if [ ! -d "$dir/.git" ]; then
    git init -q "$dir"
    git -C "$dir" remote add origin "https://github.com/$repo"
  fi
  # A tag fetches shallowly by name; a bare commit by its hash (GitHub serves both).
  git -C "$dir" fetch -q --depth 1 origin "${ref:-$commit}"
  git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
  local got
  got=$(git -C "$dir" rev-parse HEAD)
  [ "$got" = "$commit" ] || { echo "$repo: expected $commit, fetched $got" >&2; exit 1; }
}

cmake_build() { # <source dir> <build dir> [cmake args...]
  local s=$1 b=$2
  shift 2
  cmake -S "$s" -B "$b" -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$prefix" \
    -DCMAKE_INSTALL_LIBDIR=lib "$@"
  cmake --build "$b" -j "$jobs"
  cmake --install "$b"
}

license() { # <project> <files...>
  local p=$1
  shift
  mkdir -p "$prefix/share/licenses/$p"
  cp "$@" "$prefix/share/licenses/$p/"
}

cd "$src"

echo "== gmmlib 22.9.0"
fetch gmmlib intel/gmmlib 5f1c01f46ecbfceec59faef9bf3ba4625fc88fed intel-gmmlib-22.9.0
cmake_build gmmlib build-gmmlib
license gmmlib gmmlib/LICENSE.md

echo "== Level Zero loader 1.32.0"
fetch level-zero oneapi-src/level-zero d3b3efbdaa27a0aef9a9812cc8fa4260557c1e7c v1.32.0
cmake_build level-zero build-level-zero
license level-zero level-zero/LICENSE

echo "== IGC 2.28.4, with LLVM 16.0.6"
fetch igc intel/intel-graphics-compiler 6b2da2b8a2661adeaebaacbbbf7f35b5ee9e8124 v2.28.4
fetch vc-intrinsics intel/vc-intrinsics a833c8e165ebcd6c44dbb8f771ab7197fc752472 v0.24.3
fetch SPIRV-Tools KhronosGroup/SPIRV-Tools 28a883ba4c67f58a9540fb0651c647bb02883622
fetch SPIRV-Headers KhronosGroup/SPIRV-Headers 9268f3057354a2cb65991ba5f38b16d81e803692
fetch llvm-project llvm/llvm-project 7cbf1a2591520c2491aa35339f227775f4d3adf6 llvmorg-16.0.6
fetch llvm-project/llvm/projects/opencl-clang intel/opencl-clang 7161d7c6d97da80f12739cebad9bf508555bbfd9
fetch llvm-project/llvm/projects/llvm-spirv KhronosGroup/SPIRV-LLVM-Translator c13fcf6c5072483d50993e9175087bead3b7bf8a
# Opaque pointers, as Intel's v2.28.4 packages are built (its release notes: a local build defaults
# to typed pointers).
cmake_build igc build-igc \
  -DIGC_OPTION__LLVM_PREFERRED_VERSION=16.0.6 \
  -DIGC_OPTION__API_ENABLE_OPAQUE_POINTERS=ON \
  -DIGC_BUILD__VC_ENABLED=ON
license igc igc/LICENSE.md
license llvm llvm-project/llvm/LICENSE.TXT
license opencl-clang llvm-project/llvm/projects/opencl-clang/LICENSE
license spirv-llvm-translator llvm-project/llvm/projects/llvm-spirv/LICENSE.TXT
license vc-intrinsics vc-intrinsics/LICENSE.md
license spirv-tools SPIRV-Tools/LICENSE

echo "== compute runtime 26.05.37020.3"
fetch neo intel/compute-runtime a5e0dd79db5ff7b3ed6c5cd3d11064ab7cbb9aa5 26.05.37020.3
PKG_CONFIG_PATH="$prefix/lib/pkgconfig" cmake_build neo build-neo \
  -DNEO_SKIP_UNIT_TESTS=1 -DSKIP_UNIT_TESTS=1 \
  -DCMAKE_PREFIX_PATH="$prefix" \
  -DIGC_DIR="$prefix" -DGMM_DIR="$prefix" \
  -DLevelZero_INCLUDE_DIR="$prefix/include"
license compute-runtime neo/LICENSE.md

echo "== RUNPATH \$ORIGIN"
find "$prefix/lib" -maxdepth 1 -type f -name '*.so*' -print0 |
  while IFS= read -r -d '' lib; do patchelf --set-rpath '$ORIGIN' "$lib"; done
find "$prefix/bin" -maxdepth 1 -type f -perm -u+x -print0 |
  while IFS= read -r -d '' exe; do
    if file -b "$exe" | grep -q ELF; then patchelf --set-rpath '$ORIGIN/../lib' "$exe"; fi
  done

echo "== check"
for lib in libze_loader.so.1 libze_intel_gpu.so.1 libigc.so.2 libigdfcl.so.2 libigdgmm.so.12 libocloc.so; do
  [ -e "$prefix/lib/$lib" ] || { echo "missing: $prefix/lib/$lib" >&2; exit 1; }
done
if LD_LIBRARY_PATH= ldd "$prefix"/lib/*.so* 2>/dev/null | grep 'not found'; then
  echo "a library above does not resolve from $prefix/lib" >&2
  exit 1
fi
max_glibc=$(objdump -T "$prefix"/lib/*.so* 2>/dev/null | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -n 1)
echo "highest glibc symbol version needed: $max_glibc"
ocloc=$(find "$prefix/bin" -name 'ocloc*' -type f | head -n 1)
[ -n "$ocloc" ] && ln -sf "$(basename "$ocloc")" "$prefix/bin/ocloc"
"$prefix/bin/ocloc" -version 2>/dev/null | head -n 3 || true
du -sh "$prefix/lib"
