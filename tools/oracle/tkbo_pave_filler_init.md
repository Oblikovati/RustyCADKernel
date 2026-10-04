# `BOPAlgo_PaveFiller::Init` oracle

The derived probe exposes the protected native `Init` and `Clear` for exact preparation checks. It validates the resulting DS ranges, V/E iterator state, error short-circuits, locked-root non-destructive mode, preservation of an existing DS when a later call rejects null arguments, and configuration retention across `Clear`.

```sh
g++ -std=c++17 -O0 -g0 \
  -I"$OCCT_BUILD/inc" -I"$OCCT_BUILD/lin64/gcc/inc" \
  -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkbo_pave_filler_init.cxx \
  -L"$OCCT_BUILD/lin64/gcc/lib" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo \
  -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_pave_filler_init
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib" \
  /tmp/tkbo_pave_filler_init > /tmp/tkbo_pave_filler_init.expected
diff -u tools/oracle/tkbo_pave_filler_init.expected /tmp/tkbo_pave_filler_init.expected
```

This only ports protected initialization through iterator preparation. `Perform`, VV/VE/EE/VF/EF/FF geometric intersections, split-edge/block building, p-curve construction, section attributes, and inherited history/result behavior are still absent.
