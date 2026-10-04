# BOPAlgo_FaceFace precision translation

The far case applies `BOPAlgo_Tools::TrsfToPoint` to two distant face boxes, intersects the moved faces with `IntTools_FaceFace`, then applies the inverse translation to the result curve. The nearby case exercises the no-translation branch. Both records contain the translation decision, completion, curve count, curve range, and spatial endpoints.

Build against the pinned OCCT installation:

```sh
export OCCT_ROOT=/path/to/occt-build

g++ -std=c++17 -I "$OCCT_ROOT/include/opencascade" \
  tools/oracle/tkbo_face_face_task.cxx \
  -L "$OCCT_ROOT/lin64/gcc/libi" -Wl,-rpath,"$OCCT_ROOT/lin64/gcc/libi" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/tkbo-face-face-task
/tmp/tkbo-face-face-task > tools/oracle/tkbo_face_face_task.expected
```

Rust tests compare the parameter range at `1e-12` and far-origin points at `1e-7`, the representable coordinate precision at this location.

## Scope and uncovered call path

The Rust task mirrors the private wrapper's constructor state, index/face/box/tolerance setters and getters, inherited context/approximation/fuzzy settings, progress cancellation, `TrsfToPoint` decision, moved-face `Perform`, failure alert, and the separate `ApplyTrsf` phase. The fixture reproduces the wrapper's translation and result transformation using public OCCT operations because `BOPAlgo_FaceFace` is a private class inside `BOPAlgo_PaveFiller_6.cxx`.

This does not implement `BOPAlgo_PaveFiller::PerformFF`: candidate collection, edge-edge seed gathering, plane-pair filtering, per-pair option setup, parallel scheduling, and BOPDS result insertion remain outside this slice. The follow-on orchestration must consume the task without treating this fixture as end-to-end Boolean proof.
