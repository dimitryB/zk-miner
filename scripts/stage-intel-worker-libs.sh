#!/usr/bin/env bash
# Stage the shared libraries the Intel risc0 worker needs and the system does not provide, so it runs
# without LD_LIBRARY_PATH.
#
#   scripts/stage-intel-worker-libs.sh <path/to/zkminer-prove-risc0-intel> [cargo target dir]
#
# The risc0 -sys crates build their SYCL kernels as shared libraries inside the cargo target dir, and
# the worker needs oneAPI's SYCL runtime (libsycl, the Unified Runtime loader and its Level Zero
# adapters, which the loader dlopen()s). This copies all of them into `zkminer-intel/lib/` next to the
# binary, which the binary's RPATH names (see crates/zkminer-prove-risc0/build.rs). The directory is
# called `lib` because oneAPI's own libraries look for each other in `$ORIGIN/../lib`, which is then
# the same directory. Copy the binary together with `zkminer-intel/` when installing it (e.g. into
# ~/.zkminer/provers/).
#
# Needs ONEAPI_ROOT (source oneAPI's setvars.sh first). The GPU driver stack itself (Level Zero loader
# libze_loader, the compute runtime and IGC) comes from the system and is not copied.
set -euo pipefail

bin=$(readlink -f "${1:?usage: $0 <zkminer-prove-risc0-intel> [target dir]}")
[ -x "$bin" ] || { echo "not an executable: $bin" >&2; exit 1; }
target=$(readlink -f "${2:-$(dirname "$(dirname "$bin")")}")
: "${ONEAPI_ROOT:?set ONEAPI_ROOT (source oneAPI setvars.sh) first}"
oneapi=$(readlink -f "$ONEAPI_ROOT")
out=$(dirname "$bin")/zkminer-intel/lib
rm -rf "$out" && mkdir -p "$out"   # afresh: a staged copy would otherwise resolve to itself

# Where the kernel libraries are: every directory under the target dir holding a .so, the newest .so
# first, so an older build's library of the same name never wins.
mapfile -t kdirs < <(find "$target" -name '*.so' -printf '%T@ %h\n' 2>/dev/null | sort -rn |
  awk '!seen[$2]++ {print $2}')
odirs=("$oneapi/compiler/latest/lib" "$oneapi/umf/latest/lib" "$oneapi/tcm/latest/lib"
  "$oneapi/tbb/latest/lib/intel64/gcc4.8")
search=$(IFS=:; echo "${kdirs[*]}:${odirs[*]}")

# Everything ldd resolves from the target dir or oneAPI (not the system), transitively.
missing=$(LD_LIBRARY_PATH="$search" ldd "$bin" | awk '/not found/ {print $1}')
if [ -n "$missing" ]; then
  echo "unresolved even with the target dir and oneAPI on the search path: $missing" >&2
  exit 1
fi
# Each library is staged under the name it is asked for (its soname, e.g. libsycl.so.9), not the
# versioned file a symlink chain ends at (libsycl.so.9.0.0), which the loader would not look for.
n=0
stage() { # <ldd output of one object>
  while read -r name path; do
    case "$(readlink -f "$path")" in
      "$target"/*|"$oneapi"/*) cp -L "$path" "$out/$name"; n=$((n + 1)) ;;
    esac
  done < <(awk '$2 == "=>" && $3 ~ /^\// {print $1, $3}' <<<"$1")
}
stage "$(LD_LIBRARY_PATH="$search" ldd "$bin")"

# dlopen()ed, so ldd does not list them: the Unified Runtime's Level Zero adapters, and their own deps.
for lib in "$oneapi"/compiler/latest/lib/libur_adapter_level_zero*.so.[0-9]; do
  [ -f "$lib" ] || continue
  cp -L "$lib" "$out/$(basename "$lib")"; n=$((n + 1))
  stage "$(LD_LIBRARY_PATH="$search" ldd "$lib")"
done

# Loaded by name at run time too: the compute runtime's offline compiler library, which SYCL uses for
# its ahead-of-time device images. It belongs to the GPU driver stack (Ubuntu/Intel package
# `intel-ocloc`, the same version as the installed compute runtime); stage one only for a host that
# lacks it, from LIBOCLOC or the `ocloc` on PATH.
if ! ldconfig -p 2>/dev/null | grep -q 'libocloc\.so '; then
  ocloc_lib=${LIBOCLOC:-}
  if [ -z "$ocloc_lib" ] && command -v ocloc >/dev/null; then
    ocloc_lib=$(dirname "$(dirname "$(readlink -f "$(command -v ocloc)")")")/lib/x86_64-linux-gnu/libocloc.so
  fi
  if [ -n "$ocloc_lib" ] && [ -f "$ocloc_lib" ]; then
    cp -L "$ocloc_lib" "$out/libocloc.so"; n=$((n + 1))
    echo "warning: no system libocloc.so; staged $ocloc_lib (install the intel-ocloc package matching the compute runtime instead)" >&2
  else
    echo "warning: no libocloc.so on this host: install the intel-ocloc package matching the compute runtime" >&2
  fi
fi

echo "staged $n libraries in $out ($(du -sh "$out" | cut -f1))"
# The check: with no LD_LIBRARY_PATH, the binary must resolve everything through its RPATH.
if env -u LD_LIBRARY_PATH ldd "$bin" | grep -q 'not found'; then
  env -u LD_LIBRARY_PATH ldd "$bin" | grep 'not found' >&2
  echo "the binary still misses libraries: was it built with the intel feature's RPATH?" >&2
  exit 1
fi
echo "ok: $(basename "$bin") resolves all its libraries without LD_LIBRARY_PATH"
