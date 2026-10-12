# Martensite active-view node deactivation

At original main `e3fa94fbded9d7cdf7f6b60833bb63fe523a4326`, passing an
actual active or defined view-list node to `SetViewOff` takes the viewer write
guard before reading the node identity. The live reference must acquire the
same viewer's read guard, so the original regression times out after three
seconds. The compressed red log preserves that executable failure.

The fix evaluates identity before removal. Native `V3d_Viewer::SetViewOff`
intentionally owns its driver-view local across removal and deactivation;
Martensite preserves that owner. The removed active-node alias is never read
after removal. The original strong view and its distinct defined-node alias
remain valid throughout the proof.

Pinned native execution covers active-node, defined-node, and aggregate
`SetViewOff()` calls twice with nine byte-identical records. Each retains the
same defined handle slot, reduces view owners from three to two, retains one
driver-view owner, and clears activity. Rust replays those records with a real
owned embedded `V3dViewSource`, under an independent three-second bound per case.

The same-viewer API audit also checked activation, cursor/getter reads, live
view visitors, private AddView, and private DelView. Activation already reads
its alias before append. Private AddView accepts an owning handle; DelView
accepts an evaluated raw identity. The current remaining viewer view arguments
use owning handles rather than live node `HandleReference` transport. No other
write-before-node-read path was found in that bounded current-main audit.

Run `cargo test --locked -p rustycad-tkv3d --all-targets`. This regression makes
no class, constructor, renderer parity, or new compiler-coverage claim.
