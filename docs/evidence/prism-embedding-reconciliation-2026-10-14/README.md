# Prism embedding reconciliation

Source `94fe5b25b` merges main `0ceea4561` into the previously frozen candidate.
All four conflicts concerned module registration or test-only helper visibility.
Both complete-measure exports and embedding registration are retained, with the
broader fixture visibility needed by both consumers. No helper body, theorem
implementation or shared current-binding implementation changed.

The reconciled source passed 32 focused embedding/baseline tests, all-target
TKBO Clippy with warnings denied, and workspace formatting, using Rust 1.99.0 and
two build jobs. The recorded index tree and all 18 candidate crate file hashes
were identical before and after these checks. The original full frozen evidence
under `prism-embedding-v1-2026-10-14` remains unchanged and describes its original
source revision; these focused reconciliation checks do not relabel that matrix.
The coordinating agent runs the full integrated matrix separately.

No new behavior, coverage promotion, ThreadSanitizer or release completion claim.
