# ShapeFix_Wire FixSmall oracle

This focused probe uses the pinned OCCT checkout at commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`. Its three-edge wire contains a
short edge whose vertices remain distinct under `BRepBuilderAPI_MakeEdge`.
The native and Rust fixtures use coordinates `(0,0,0)`, `(1e-5,0,0)`, and
`(1,0,0)`, and pass `2e-5` as `precsmall`.

Build against the installed pinned OCCT libraries:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/shape_fix_wire_small.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/rustycad-shape-fix-wire-small-oracle
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib:/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/rustycad-shape-fix-wire-small-oracle
```

Pinned native output:

```text
small_edge topo=0 changed=0 edges=3 done2=0 fail2=1 fail3=0
small_edge topo=1 changed=1 edges=2 done2=1 fail2=0 fail3=0
```

The fixture specifically avoids a `1e-8` edge: OCCT's edge builder merges
its endpoints at construction, so that edge does not exercise `DONE2`.
Rust regressions are in `tests/shape_fix_wire_small.rs`.
