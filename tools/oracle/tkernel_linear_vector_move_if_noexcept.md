# LinearVector relocation dispatch

The pinned OCCT `grow()` path relocates each existing non-trivial element with
`std::move_if_noexcept`. A type with a potentially throwing move constructor
and a copy constructor therefore observes copies during `Reserve`; Rust's
`Vec` relocation does not call `Clone`. The focused native probe prints
`1 0 1 42`, while the supplemental Rust regression confirms its clone count
stays zero and its value remains 42. The Rust collection retains the requested
capacity rule and value state, but this user-defined constructor side effect
remains a behavior gap.

The paired exception probe checks native logical size and already-written
common assignment values when copy construction throws during `Resize` or
copy assignment. Rust now restores logical length and drops constructed tail
values through a guard. Native leaves the same published size and common value,
but the C++ grow/construct code leaks untracked allocations or objects after
the exception. Rust cleanup is intentionally safer and is not an exact
resource-lifetime match.

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Reproduce both probes from the repository root:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/OCCT/src/FoundationClasses/TKernel/NCollection \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_linear_vector_move_if_noexcept.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel_linear_vector_move_if_noexcept
/tmp/tkernel_linear_vector_move_if_noexcept

g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/OCCT/src/FoundationClasses/TKernel/NCollection \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_linear_vector_exception_state.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel_linear_vector_exception_state
/tmp/tkernel_linear_vector_exception_state
```
