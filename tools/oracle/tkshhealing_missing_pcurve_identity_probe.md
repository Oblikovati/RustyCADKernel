# TKShHealing missing-p-curve handle probe

This probe checks whether `ShapeUpgrade_UnifySameDomain::Build` replaces a
valid source p-curve handle for the two pinned missing-p-curve fixtures. The
fixture remains the OCCT test source at `3d097a0328e71b826377d4814ab05ec3c3d23871`
with SHA-256 `2214339e0275b89018a09ca6cbd8e01e610614eacd4869f17f8001049db538a1`.

The OCCT checkout must remain clean. Create a disposable worktree and apply the
instrumentation patch there:

```sh
git -C ../OCCT worktree add --detach /tmp/occt-tksh-pcurve-probe \
  3d097a0328e71b826377d4814ab05ec3c3d23871
git -C /tmp/occt-tksh-pcurve-probe apply \
  /path/to/RustyCADKernel/tools/oracle/tkshhealing_missing_pcurve_identity.patch
```

Configure that copy with the same OCCT modules and test options used by the
local native build, build `OpenCascadeGTest`, then run:

```sh
ctest --test-dir /tmp/occt-tksh-pcurve-build \
  -R 'ModelingAlgorithms::TKShHealing::ShapeUpgrade_UnifySameDomainTest\.(MissingPCurveDoesNotAbortShellProcessing|MissingSourcePCurveDuringTransformation)$' \
  -V
```

The patch records a valid edge's `CurveOnSurface(..., ReferenceFace)` handle
before and after `Build`. Pointer values vary per process; compare each pair for
equality. The pinned fixtures reported equality in both tests, and both native
tests passed. This observation is specific to the equal-cylinder-support
fixtures and does not establish the broader `TransformPCurves` behavior.
