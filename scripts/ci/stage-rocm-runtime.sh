#!/usr/bin/env bash
# Stages the HIP runtime the release's AMD workers load, so a host needs only the amdgpu kernel driver:
#
#   stage-rocm-runtime.sh <ROCm root, e.g. /opt/rocm/core-10.1> <out dir>
#
# <out dir>/lib gets everything libamdhip64.so.7 loads from the ROCm install, transitively, at the same
# path relative to <ROCm root>/lib: AMD's ROCm 10.x libraries find each other through RUNPATHs like
# $ORIGIN/llvm/lib and $ORIGIN/rocm_sysdeps/lib, so the layout has to be kept. Only glibc and libstdc++
# come from the host (ROCm 10.1 needs glibc 2.27, GLIBCXX 3.4.22). <out dir>/LICENSES gets the
# licenses of what was staged.
#
# The risc0 ROCm worker finds lib/ through its RPATH ($ORIGIN/zkminer-rocm/lib); the SP1 HIP server
# through the library path the SP1 worker gives it (`sp1-gpu-server-hip-lib`, a link to lib/).
set -euo pipefail

root=$(readlink -f "${1:?usage: $0 <ROCm root> <out dir>}")
out=${2:?usage: $0 <ROCm root> <out dir>}
lib=$root/lib
rm -rf "$out" && mkdir -p "$out/lib" "$out/LICENSES"

# The closure, as the loader resolves it with no LD_LIBRARY_PATH: each library's own RUNPATH.
mapfile -t deps < <(env -u LD_LIBRARY_PATH ldd "$lib/libamdhip64.so.7" |
  awk '$2 == "=>" && $3 ~ /^\// {print $1, $3}')
staged=0
copy() { # <name the loader asks for> <path it resolved to>
  local name=$1 path=$2 rel
  case "$(readlink -f "$path")" in "$lib"/*) ;; *) return 0 ;; esac
  # Where the name was found, relative to lib/: its directory, and the name itself (a soname link
  # becomes a file of that name).
  rel=$(realpath --relative-to="$lib" "$(dirname "$path")")
  mkdir -p "$out/lib/$rel"
  cp -L "$path" "$out/lib/$rel/$name"
  staged=$((staged + 1))
}
copy libamdhip64.so.7 "$lib/libamdhip64.so.7"
for d in "${deps[@]}"; do copy ${d}; done
if env -u LD_LIBRARY_PATH ldd "$lib/libamdhip64.so.7" | grep -q 'not found'; then
  echo "libamdhip64.so.7 does not resolve within $root" >&2
  exit 1
fi

# The staged copy must resolve within itself.
if env -u LD_LIBRARY_PATH ldd "$out/lib/libamdhip64.so.7" | grep -E 'not found|=> /opt/rocm'; then
  echo "the staged runtime does not resolve within $out/lib" >&2
  exit 1
fi

# Licenses. AMD's packages carry the texts of HIP, ROCr, comgr and rocprofiler-register; for the
# rest (LLVM, and the system libraries AMD builds into rocm_sysdeps/) the standard texts come from
# the build image's /usr/share/common-licenses, and README maps each library to its license and
# names where AMD publishes the sources (the LGPL ones need that).
for component in hip rocr amd_comgr rocprofiler-register; do
  [ -d "$root/share/doc/$component" ] && cp -r "$root/share/doc/$component" "$out/LICENSES/"
done
for text in Apache-2.0 LGPL-2.1 LGPL-3 GPL-2 GPL-3; do
  [ -f "/usr/share/common-licenses/$text" ] && cp "/usr/share/common-licenses/$text" "$out/LICENSES/"
done
version=$(basename "$root")
cat > "$out/LICENSES/README" <<EOF
zkminer-rocm/lib is AMD's ROCm HIP runtime ($version) and what it loads, copied unmodified from AMD's
Ubuntu 22.04 packages (https://stable.repo.amd.com/rocm/core/packages/ubuntu2204/). None of it is
covered by zkminer's MIT license. Each library is under its own license:

  libamdhip64.so.7              HIP runtime: MIT (hip/LICENSE.md)
  libhsa-runtime64.so.1         ROCr: University of Illinois/NCSA (rocr/LICENSE.md)
  libamd_comgr.so.3             code object manager: Apache-2.0 WITH LLVM-exception (amd_comgr/)
  librocprofiler-register.so.0  MIT (rocprofiler-register/LICENSE.md)
  librocm_kpack.so.0            AMD ROCm kpack (no license file in the packages; see ROCm/TheRock)
  llvm/lib/libLLVM, libclang-cpp  LLVM: Apache-2.0 WITH LLVM-exception (Apache-2.0)
  rocm_sysdeps/lib/
    librocm_sysdeps_elf         elfutils libelf: LGPL-3.0-or-later OR GPL-2.0-or-later (LGPL-3, GPL-2, GPL-3)
    librocm_sysdeps_numa        numactl libnuma: LGPL-2.1 (LGPL-2.1)
    librocm_sysdeps_drm*        libdrm: MIT
    librocm_sysdeps_z           zlib: Zlib
    librocm_sysdeps_zstd        zstd: BSD-3-Clause
    librocm_sysdeps_bz2         bzip2: bzip2-1.0.6
    librocm_sysdeps_liblzma     xz liblzma: 0BSD

The sources of all of them, as AMD built them, are published at https://github.com/ROCm/TheRock
(the build system of these packages, with each library's source revision) and https://github.com/ROCm.
The LGPL libraries are loaded dynamically and may be replaced by a compatible build.
EOF

echo "staged $staged libraries in $out/lib ($(du -sh "$out/lib" | cut -f1)):"
(cd "$out/lib" && find . -type f | sort)
