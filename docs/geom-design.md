# Geom2d and Geom class hierarchy in Rust

This note fixes how the transient geometry packages of ModelingData (`Geom2d` in
TKG2d now, `Geom` in TKG3d next, with the same rules) map to Rust. The first
port is `crates/tkg2d` (`rustycad-tkg2d`).

## 1. Classes

| OCCT | Rust |
| --- | --- |
| abstract class with virtual methods (`Geom2d_Geometry`, `Geom2d_Curve`, `Geom2d_BoundedCurve`, `Geom2d_Conic`, `Geom2d_Point`, `Geom2d_Vector`) | trait with the same name and a `Transient` supertrait chain (`trait Geom2dCurve: Geom2dGeometry`) |
| concrete class (`Geom2d_Circle`, ...) | struct `Geom2dCircle` that implements every trait of its ancestry |
| pure virtual / overridable member | required or provided trait method; a provided method that no class overrides is the native non-virtual member (`Mirror`, `Reversed`, `Value`, `D0`...) |
| protected data of an abstract class (`Geom2d_Conic::pos`, `Geom2d_Vector::gpVec2d`) | stored in each concrete struct; the trait reads and writes it through two required accessors, and its provided methods hold the shared logic once |
| `occ::handle<Geom2d_Curve>` | `Handle<dyn Geom2dCurve>` |
| `occ::handle<Geom2d_Circle>` | `Handle<Geom2dCircle>` |
| copy constructor (`new Geom2d_BSplineCurve(*c)`) | `Clone` with the native copy semantics (for example the resolution cache is reset) |

Names keep the package prefix (`Geom2dCircle`, not `Circle`), because `gp`
already has `Circle2`, and Geom will add its own `GeomCircle`.

## 2. Handles, RTTI and down casts

The TKernel `Handle<T>` (crates/tkernel/src/standard/handle.rs) owns an
`Arc<dyn Transient>` and projects it to `T`. Before this port, `T` could only be
a struct reached through embedded parent fields, which cannot express virtual
dispatch. The handle is extended, not replaced:

- `Handle<T: ?Sized>`: `T` can be a trait object.
- `RuntimeTyped` gets `project_from(&dyn Transient) -> Option<&Self>`. Structs
  implement it with the existing projection and downcast; trait objects with a
  lookup in the dynamic descriptor.
- `RuntimeType::with_interfaces` stores, for each abstract ancestor trait, an
  `InterfaceCaster<dyn Trait>`: a plain `fn(&dyn Transient) -> Option<&dyn Trait>`
  that the concrete class generates (`impl_transient!(..., interfaces: [...])`).
  It downcasts to the concrete type and coerces. There is no `unsafe` and no
  global registry: the caster lives in the static descriptor of the class.
- `impl_transient_interface!(dyn Geom2dCurve, "Geom2d_Curve", dyn Geom2dGeometry)`
  gives each abstract class its own descriptor, so `DynamicType`, `IsKind`,
  `IsInstance` and names follow the native chain
  `Geom2d_Circle -> Geom2d_Conic -> Geom2d_Curve -> Geom2d_Geometry -> Standard_Transient`.

`occ::down_cast<Geom2d_Circle>(h)` is `h.cast::<Geom2dCircle>()`, and an up cast
is the same call. A failed cast returns a null handle, as in OCCT. Handle
equality compares the owned object, whatever `T` is. Dereferencing a handle
gives `&T`, so `curve.eval_d1(u)` dispatches through the trait object.

## 3. Shared mutation

In OCCT, `c->SetRadius(5)` is visible through every handle that shares the
object. A Rust handle only gives `&T`, so each geometry object keeps its mutable
state behind interior mutability:

- Each concrete struct keeps all its fields in one `SharedState<State>`, the
  TKernel primitive `rustycad_tkernel::standard::SharedState` that every
  toolkit's transients share (the TKMath Poly transients use it too).
  It is a thin wrapper around `std::sync::RwLock` that ignores lock poisoning
  (the port does not panic while a lock is held). Every native member, setters
  included, takes `&self`. Evaluators take a read lock and setters a write lock, so readers on
  several threads do not block each other, and objects stay `Send + Sync`.
- One lock per object keeps each call atomic. The algorithm of a class is
  written on its plain `State` value; the struct only takes the lock and calls
  it. For example `MovePoint` evaluates the curve on the state that it already
  holds for writing, and does not call back through a handle.
- A method never holds its own lock while it calls another handle, because the
  other handle can be the same object (`Geom2d_OffsetCurve::SetBasisCurve` reads
  offsets of nested curves). It reads what it needs, releases, computes, then
  writes. A basis curve that a trimmed or offset curve owns is a separate object
  with its own lock.
- Getters that return `const T&` in OCCT (`Poles()`, `Position()`) return
  copies, because a reference cannot outlive the lock. `WeightsArray()`, whose
  identity the legacy tests check, lends the stored array to a closure instead.
- Concurrent mutation of one object from several threads is memory safe, and
  each call is atomic, but a sequence of calls is not. OCCT has no
  synchronisation at all here, so the port adds no stronger promise.

A handle-level lock (`Handle<RwLock<T>>`) was rejected: it cannot give the
per-object lock of a trait object without projecting guards, and it makes every
evaluation spell out a lock that the native API does not have.

## 4. Exceptions

A member that can throw returns `Result<_, Failure>`. The failure carries the
native exception type (`Geom2d_UndefinedDerivative`, `Geom2d_UndefinedValue`,
`Standard_ConstructionError`, `Standard_RangeError`, `Standard_OutOfRange`,
`Standard_DomainError`, `Standard_NoSuchObject`...) and a message with the
offending value. Checks that OCCT writes with `*_Raise_if` stay active, because
the pinned oracle build (RelWithDebInfo) does not define `No_Exception`. The
virtual evaluators (`EvalD0`...`EvalDN`) and `ReversedParameter` always return
`Result`, because an override can fail even when the base class cannot (the
Geom2dEval curves raise Standard_NotImplemented from `ReversedParameter`). For the same reason `Copy`
returns `Result`: a trimmed curve re-trims its bounds when it is copied.

## 5. Evaluation caches

In OCCT 8.1.0-dev1, `Geom2d_BSplineCurve` and `Geom2d_BezierCurve` no longer
keep a `BSplCLib_Cache`: they evaluate through `BSplCLib::D0..DN` directly. The
only lazy value is `myMaxDerivInv` of `Resolution`, which OCCT computes in a
non-const member. The port keeps it in the class state, so `resolution(&self)`
takes the write lock once, fills the value and reads it. Every mutator that
invalidates it in OCCT clears the flag in the same state. Geom (3D) adds
surface caches with the same rule: the cache is part of the locked state and
is filled under the write lock.

## 6. Evaluation representations

`Geom2d_BezierCurve`, `Geom2d_BSplineCurve` and `Geom2d_OffsetCurve` accept an
optional `Geom2dEval_RepCurveDesc` that redirects evaluation to another curve.
The descriptor and `Geom2dEval_RepUtils.pxx` are header-only parts of the
`Geom2dEval` package in TKG2d that `Geom2d` includes. They are ported in
`geom2d_eval::rep_curve_desc` with the Geom2d classes.

## 7. Adaptors

`Adaptor2d_Curve2d` follows the same rules: a trait `Adaptor2dCurve2d` with a
`Transient` supertrait, reached through `Handle<dyn Adaptor2dCurve2d>`, with one
struct per concrete adaptor (`Adaptor2dLine2d`, `Adaptor2dOffsetCurve`,
`Geom2dAdaptorCurve`). Unlike the Geom2d abstract classes, the native base class is
not abstract: every member raises Standard_NotImplemented except `NbSamples`, so
every trait member returns `Result` and has that default, and
`Adaptor2dCurve2dBase` is an instance of the base class itself.

`Geom2dAdaptor_Curve` keeps the curve kind, its gp geometry and the `BSplCLib_Cache`
of Bezier and B-spline curves (`CurveDataVariant`) in its locked state. The native
evaluators are `const` and fill the mutable cache; the port takes the write lock for
`EvalD0` to `EvalD3` and a read lock elsewhere. An offset curve holds a nested
adaptor on its basis, which is a separate object with its own lock.
## 8. Eval curves and hashers

The Geom2dEval curves (`Geom2dEvalSineWaveCurve`, `Geom2dEvalTBezierCurve`...) follow
the same class mapping, but no mutator succeeds on them (`Reverse` and `Transform`
raise Standard_NotImplemented), so they keep plain fields without a
`SharedState`. The Geom2dHash structs keep their public `CompTolerance` and
`HashTolerance` as fields; `operator()(key)` becomes `hash` and
`operator()(key, key)` becomes `equal`, and the value and curve hashers also implement
the NCollection `HashPolicy`, so they can drive the ported hashed containers.

## 9. Grid evaluators

The Geom2dGridEval evaluators are plain structs, not transient classes: each holds the
handle it evaluates (`Handle<Geom2dCircle>`...), may hold a null one, and reads the
curve once per call (`Circ2d()`, the pole and knot arrays), so evaluation takes the
curve's lock only for those reads. The `std::variant` of evaluators in `Geom2dGridEval_Curve` is the
enum `Geom2dGridEvalEvaluator`, with `Uninitialized` for `std::monostate`, and the
adaptor fallback borrows its adaptor with a lifetime, as the native
`std::reference_wrapper`. The result structs `Geom2dGridEval::CurveD1` to `CurveD3` are
aliases of `Geom2dCurveD1` to `Geom2dCurveD3`, which have the same fields.
