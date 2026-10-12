# Martensite inventory lookup optimization

The full crosswalk report was compared against the same compiler and ledger input
files before and after narrowing repeated lookups. All report fields are identical
except `tool_implementation_sha256`, which correctly records changed tooling.
Coverage, ambiguity, parent USR checks, declaration kind checks, input hashes and
proof gaps are preserved. This historical input comparison does not establish
fresh current-main behavioral completeness.

The local cProfile runs took 314.379 seconds before and 90.134 seconds after.
Constructor lookup now filters qualified-name groups before inspecting declarations;
Rust kind lookup filters only the targets requested by each row. Actual constructor
parent and kind validation still runs, preserving duplicate candidates and order.
Named fake indexes reject unrelated scans in regression tests.

These are descriptive timings with concurrent local work, not isolated performance
baselines. Raw profiles remain in `target/api-crosswalk-profile.pstats` and
`target/api-crosswalk-optimized-profile.pstats`; no build artifacts are archived.
