# Martensite coverage workload evidence

GitHub run 38156653032 timed out after 300.014 seconds in
`BVH_PairDistanceTest__Bug30655_SphereDistance`. All functional matrices passed.
The unchanged regression meshes 100,520 and 110,588 triangles. A local instrumented
trace spent 46.3 seconds meshing, 1.5 seconds building BVHs and 118.4 seconds in
the distance query. This is a dense workload repeated by the coverage job.

Coverage now excludes exactly this additional instrumented replay. The unchanged
regression remains mandatory in the Linux, Windows and macOS functional toolkit
suites. OCCT remains a local oracle only. Neither test inputs, Rust implementation,
build profiles nor timeout limits changed.

The exact revised workflow command passed all 17,083 selected tests in 170.579
seconds, with one skipped test. The JUnit report contains 17,083 cases, no failures
or errors, and no sphere-distance case. LCOV output is nonempty. The retained
compressed logs include the original CI failure, local workload trace and complete
filtered workspace verification. CI run 38157888201 independently completed green
before this change.

The workflow regression checks preserve exactly one coverage invocation and one
precise exclusion while retaining unfiltered functional suites. The JUnit artifact
path is `target/nextest/ci/junit.xml`, matching the produced report.

All 199 tools-test modules pass using the CI command
`python3 tools/run_tool_tests.py --jobs 2`; 13 toolkit/workflow regressions pass.
