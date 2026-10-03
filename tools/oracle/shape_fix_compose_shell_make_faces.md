# ShapeFix_ComposeShell::MakeFacesOnPatch nested-loop oracle

Pinned to OCCT source revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The fixture gives ComposeShell one planar patch, an outer square, and a reversed
contained square. It records `Perform`, face count, and wire count to cover the
root/hole assembly path.

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/shape_fix_compose_shell_make_faces.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKernel \
  -o /tmp/shape_fix_compose_shell_make_faces
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib:\
/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/shape_fix_compose_shell_make_faces
```

The native probe requires an initialized `ShapeBuild_ReShape` context before
`Perform`; without it the pinned implementation dereferences a null context.
