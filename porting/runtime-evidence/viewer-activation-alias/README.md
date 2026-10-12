# Martensite viewer activation alias correction

The original aggregate SetViewOn reproducibly timed out with one defined mapped
view and zero lights. Its per-view append evaluated the viewer write-lock receiver
before reading the argument from a node in that same viewer. The correction reads
the genuine live argument first, then moves that single handle into the active
list. No handle snapshot survives the subsequent native virtual callbacks.

The bounded Rust regression uses an actual embedded view subclass and real
inherited redraw, and replays three pinned native slot-identity, view-owner and
driver-owner rows. Both native runs are byte-identical. The preserved minimized
original failure is intentional before-fix evidence; final tests pass.

This method-call order differs from assignment's RHS-first order, already proved
correct in the separate camera self-assignment regression. Both SetViewOn rows
remain partial until the separate actual live-light callback family is proved.
No whole-class/constructor credit or global ledger changes are made here.

Run `cargo test --locked -p rustycad-tkv3d --all-targets`.
