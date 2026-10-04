# PerformFF stage oracle

This probe constructs a bounded plane and a bounded cylinder, then runs native
`BOPAlgo_PaveFiller::Perform`. It prints the face/face records, the first section
curve range and stored box/tolerance, followed by the final face `VerticesOn`
and `VerticesIn` extents. The tracked output is `tkbo_perform_ff.expected`.

The probe also creates a fresh `BOPDS_DS`, calls `Init`, and runs the same
`UpdateFaceInfoOn`/`UpdateFaceInfoIn` calls over face indices 0 and 10. The
Rust `perform_ff` slice matches that stage-only face state (4/0 and 2/0) and
the section record and expanded curve box. Full native `Perform` later reports
3 `VerticesOn` entries on the second face because preceding VF/EF result
producers have augmented it. This stage alone does not compare to the complete
pipeline. EE-generated seam-shift points and EF-generated walking seeds return
`Standard_NotImplemented` explicitly.

Native source order at `BOPAlgo_PaveFiller_6.cxx:560-573` bases `AddInterf`
on raw intersector `NbCurves`/`NbPoints`, before `IntTools_Tools::CheckCurve`
can filter a thin curve out of `BOPDS_InterfFF`. Rust uses that raw-count
predicate; a regression supplies a default invalid curve and checks that curve
storage rejects it while the pair interference remains present.

Build against the pinned local OCCT installation:

```sh
c++ -std=c++17 -O0 -I"$OCCT_INCLUDE" tools/oracle/tkbo_perform_ff.cxx \
  -L"$OCCT_LIB" -lTKBO -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -Wl,-rpath,"$OCCT_LIB" -o /tmp/tkbo_perform_ff
/tmp/tkbo_perform_ff
```

`OCCT_INCLUDE` and `OCCT_LIB` point to the build's `include/opencascade` and
`lin64/gcc/libi` directories. The tool inspects results only; it does not alter
the OCCT implementation.
