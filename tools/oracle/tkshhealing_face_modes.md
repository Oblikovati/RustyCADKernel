# Face repair modes

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Three rows compare all eleven face modes at construction, after assignment of
3, and after ClearModes. The shared wire topology mode remains true after
ClearModes, which resets only the face's modes. This verifies scalar configuration;
ShapeFixFace::Perform is still incomplete for the corresponding repair stages.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_face_modes`.
