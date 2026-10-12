# Martensite borrowed light-name options

`V3dImageDumpOptions<'anchor>` now stores the actual nullable C pointer through
`V3dLightName<'anchor>`. Copying options copies that pointer; it does not allocate,
own, encode or clone its bytes. The public aggregate uses native C field order,
and the pointer wrapper has transparent representation.

Borrow an immutable owner safely:

```rust
use rustycad_tkv3d::v3d::{V3dImageDumpOptions, V3dLightName};
let owner = std::ffi::CString::new("sun").unwrap();
let options = V3dImageDumpOptions {
    light_name: V3dLightName::from_c_str(&owner),
    ..Default::default()
};
let copied = options;
assert_eq!(copied.light_name.as_ptr(), owner.as_ptr());
```

The compiler rejects a safe borrowed name that outlives its owner. Native default
is a non-null static empty string; `V3dLightName::null()` preserves a distinct null
pointer. `unsafe V3dLightName::from_ptr` is the narrow foreign-storage seam for
externally maintained buffers: the caller anchors allocation lifetime and
NUL-termination, synchronizes edits with reads and never reallocates/releases
storage during consumption. It reads nothing at construction. No unsafe lifetime
erasure, fabricated owner, persistent lock or invented Send/Sync implementation
is involved. Dangling or racing reads remain invalid native/Rust uses.

Native `V3d_View::ToPixMap` dereferences the name only for its untiled ShadowMap
branch, after virtual Redraw, when constructing `TCollection_AsciiString`. The
Martensite consumer now does the same. It copies bytes through the first NUL at
that instant, retaining arbitrary non-UTF8 values. Null produces typed
`Standard_NullObject` after redraw and before normal state cleanup. Other buffer
kinds do not read the pointer; empty and named strings retain ordinary backend
shadow-map lookup behavior.

Pinned OCCT 3d097a0 assets prove thirteen pointer/value/layout rows: default and
null, original/copy aliases, shared external edits, shortening at a new NUL,
rebind without retargeting the old copy, non-UTF8/interior/empty pointers and all
nine aggregate field offsets. Earlier fifteen rows continue to cover every
named buffer/stereo variant and the eight scalar fields. The Linux host ABI
layout receipt is platform-specific; Windows/macOS remain deferred.

A second actual native ToPixMap probe uses the shared named null backend.
Three Rust public ToPixMap cases use the existing named recording backend and
match exception/redraw/layer-state observations: RGB-null is not dereferenced,
ShadowMap-null fails after one redraw with target42/single-layer state retained,
and non-UTF8 ShadowMap input completes the name conversion and ordinary cleanup.
These gates do not compare the two backends' pixels or bool return values, and
do not claim ToPixMap/backend-family completion. Native probes run locally;
ordinary tests replay committed assets using Rust only.

The actual callback probe additionally changes the anchored buffer `sun` to
`moon` in Redraw. ShadowMapDump observes the freshly created `moon` byte string,
changes the original anchor to `star`, and still sees `moon` in its own argument.
The final options pointer reads `star`. Native/Rust public virtual chains and
Miri agree, so this proves consumption timing with a real byte observer.

ToPixMap remains partial: native Redraw can also rebind `options.LightName`
through another valid nonconst alias to the original options object. Rust's
current `&V3dImageDumpOptions` consumer does not provide that mutable-object
reference transport. The rooted native rebind proof is separate from these
complete public POD field/copy semantics; no view-family credit follows here.
