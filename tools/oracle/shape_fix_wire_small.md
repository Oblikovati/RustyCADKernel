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
connected topo=0 changed=1 edges=2 done1=0 done2=0 done3=1 shared=1
connected topo=1 changed=1 edges=2 done1=0 done2=0 done3=1 shared=1
connected_all changed=1 status_done=1 shared=1
degenerated_small changed=1 edges=1 done1=1 fail1=1
connected_single topo=0 changed=1 done3=1 shared=1
connected_single topo=1 changed=1 done3=1 shared=1
```

The short-edge fixture avoids a `1e-8` edge: OCCT's edge builder merges its
endpoints at construction, so that edge does not exercise `DONE2`. The two-edge
connection case uses a `1e-4` gap and a `1e-3` check tolerance, so native chooses
`DONE3`; both modes replace both working edge endpoints with one shared vertex.
The aggregate `FixConnected(precision)` regression also confirms its native
boolean result, accumulated `StatusConnected(DONE)`, and updated shared endpoint.
Rust regressions are in `tests/shape_fix_wire_small.rs`.
