# Joined orientation main integration

Main source `0d2bef9d9974ba157761bbf8a1bcc272e5393c34` contains candidate
`660a1bbaee30f4a8338890bd1c24bca8f994a0cf` without candidate source changes.

The root reran TKBO all-target/all-feature tests: **1,413 passed, zero failed or
ignored**, plus **35 documentation tests**, strict Clippy, workspace formatting,
and warning-denied generated documentation. All 1,413 candidate test names remain
present and passing. All 30 candidate files and source bookends match.

Three outer and 76 archived payload hashes were verified. Both candidate source
and sanitizer bookends agree. Two new main processes reproduce the complete
215,177-byte owned orientation receipt exactly (SHA-256
`d642f0f81b03da8689cd7930d2fa27b269faec055d9629bdb507422bcd321968`).
The legacy 752-byte scalar and 1,949-byte callback fixtures remain unchanged and
their comparison tests pass. Commands, platform and logs are retained alongside
`integrated-main.json` and `integrated-supplement.json`.

This accepts conditional orientation evidence bound to one immutable prism join.
It does not accept a complete body, public operation, application or release.
ThreadSanitizer remains incomplete: the candidate instrumented executable crashed
before test output on the ordinary host. No host protection bypass was applied.
