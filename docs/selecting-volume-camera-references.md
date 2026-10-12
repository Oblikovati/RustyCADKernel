# Martensite selecting-volume camera references

The selecting-volume camera setter family accepts a genuine
`&dyn HandleReference<Graphic3dCamera>` argument. Existing owning handles remain
valid arguments, while a view camera field reference observes native-valid
replacement during an arbitrary virtual intersector setter callback.

For example, `manager.set_camera(view.camera_ref().as_ref())?` preserves the
original camera field identity. The actual intersector and builder fields acquire
ownership at native assignment points. The manager catches typed Martensite
`Failure` transport through the unchanged void virtual setter and resumes
unrelated panics; ordinary failures still propagate.

Six source-pinned native CPU rows and Rust-only replay cover alias identity,
exact intrusive counts, replacement, failure, unrelated exception, and retry.
Real embedded rectangular geometry also exercises build/copy/overlap delegation.
No OCCT bindings or GPU are required by the Rust tests.

The selector Pick callback orchestration remains in the separate unfinished
Frame branch. Copied camera getter conveniences and full selecting-volume or
controller class contracts remain partial; this checkpoint credits setters only.
