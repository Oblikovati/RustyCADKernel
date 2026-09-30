// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 declares this member without defining it.
// Keep this separate from runnable probes: successful compilation plus this exact linker
// failure records an upstream executable-API gap, not a completed Rust implementation.
#include <IntTools_PntOnFace.hxx>
int main() { return IntTools_PntOnFace().IsValid(); }
