# Shared frozen chart linking visibility

Source parent: `df830f29e`. Only visibility, reexports and documentation change;
existing function bodies, call paths, fields and encoding order are unchanged.
The helpers remain within the frozen subtree and expose no authority constructor.

The actual authored annular capture test passed. Seven fresh process fixtures
passed with all full payloads exactly equal to the pre-change baseline (including
internal repeated/thread comparisons). `structural.json` records lengths/digests;
compressed logs retain complete payloads and commands are in the script.
No broad ordered-validation or body-validity acceptance is inferred.
