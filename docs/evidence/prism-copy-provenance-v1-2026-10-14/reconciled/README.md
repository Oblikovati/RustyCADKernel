# Reconciled copy lineage source checks

Source: `b05bc6255bbfdcebb8c19b31dd26b4f6fe35dc8a`.
This is a clean merge of main `a3245eace` into the existing candidate, following
the structural borrowed-view commit `c39731488`. Historical source and evidence
commits remain reachable. No fixture or replay expectation changed.

`final-receipt.json` records focused lineage/retention tests, two independent
lineage process replays, the legacy frozen replay, three-package all-target
all-feature Clippy, and formatting. Full tracked crate/Cargo source bookends
are retained. The full 6,066-test / 302-doctest matrix in the parent evidence
directory ran on earlier `cc3efbf37`, before the views and merge. The coordinator
will run the full integration matrix on actual merged main source.

Replay summaries require exact equality to the earlier lineage and legacy
bytes. The inspection remains private diagnostic lineage with historical selected
geometry and unresolved partition/deletion/whole-body validation obligations;
these checks do not turn it into a successful body or sanitizer acceptance.
