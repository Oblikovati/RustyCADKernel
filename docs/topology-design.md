# TopoDS, TopExp and TopTools in Rust

This note fixes how the topology packages of TKBRep (`TopoDS`, `TopExp`,
`TopTools`) map to Rust in `crates/tkbrep` (`rustycad-tkbrep`). It extends
[geom-design.md](geom-design.md): handles, RTTI, shared mutable state and
exceptions follow that note, and this one only covers what Geom does not have.
The TopLoc package that the shapes use is the TKMath port
(`rustycad_tkmath::toploc`), reused as it is.

## 1. TopoDS_Shape is a value

`TopoDS_Shape` is not a transient. It is a small value with three fields:

| OCCT field | Rust field | Meaning |
| --- | --- | --- |
| `occ::handle<TopoDS_TShape> myTShape` | `Handle<dyn TShape>` | the shared topology (null for a null shape) |
| `TopLoc_Location myLocation` | `toploc::Location` | the placement of this use of the TShape |
| `TopAbs_Orientation myOrient` | `top_abs::Orientation` | the orientation of this use |

`topods::Shape` derives `Clone` (a handle copy and a location copy, as the C++
copy constructor). A default shape is null, with the identity location and the
`EXTERNAL` orientation, as `TopoDS_Shape()`. Setters (`set_location`,
`set_orientation`, `reverse`, `move_by`...) take `&mut self` and change only this
value; `located`, `oriented`, `reversed`, `moved`... return a changed copy, as the
native `Located`, `Oriented`... members do. The OCCT member names that are both a
getter and a setter (`Location()` / `Location(L)`, `Free()` / `Free(bool)`) become
`location` / `set_location`, `free` / `set_free`, and so on.

The typed shapes `TopoDS_Vertex`, `TopoDS_Edge`, `TopoDS_Wire`, `TopoDS_Face`,
`TopoDS_Shell`, `TopoDS_Solid`, `TopoDS_CompSolid` and `TopoDS_Compound` add no
field natively. In Rust each one is a newtype over `Shape` (`topods::Vertex`...)
with `Deref`/`DerefMut` to `Shape` and `From<Vertex> for Shape`, so every `Shape`
member is available on it and a typed shape goes wherever a `&Shape` is expected.
The builder takes the typed shape it makes (`make_wire(&mut Wire)`), as natively.

The casts of the `TopoDS` namespace (`TopoDS::Vertex(S)`...) check the type
(`Standard_TypeMismatch` when the shape is not null and has another type) and
return a typed copy: a Rust reference cannot be reinterpreted as another type
without `unsafe`. The non-const overloads, which return a `TopoDS_Vertex&` alias
of the argument, therefore also return a copy: write the modified copy back when
the original must change. The TopExp functions use the unchecked cast, because
`TopExp.cxx` defines `No_Standard_TypeMismatch` (see section 7).

## 2. The TShape hierarchy

`TopoDS_TShape` is abstract (its `EmptyCopy` is pure virtual), so it is the trait
`topods::TShape` with a `Transient` supertrait, registered as `TopoDS_TShape` by
`impl_transient_interface!`. The shared state of the base class (the child list
and the 16-bit state word) is the struct `TShapeBase`; each concrete class embeds
it and the trait reads it through the required accessor `tshape_base()`. Every
flag getter and setter, `ShapeType` and `NbChildren` are provided trait methods
written once over that base, as the non-virtual native members are.

| OCCT | Rust | Native constructor defaults |
| --- | --- | --- |
| `TopoDS_TVertex` (abstract) | `TVertex` struct, not a `TShape` | Closed, Convex |
| `TopoDS_TEdge` (protected constructor) | `TEdge` | none |
| `TopoDS_TWire` | `TWire` | none |
| `TopoDS_TFace` | `TFace` | none |
| `TopoDS_TShell` | `TShell` | none |
| `TopoDS_TSolid` | `TSolid` | not Orientable |
| `TopoDS_TCompSolid` | `TCompSolid` | none |
| `TopoDS_TCompound` | `TCompound` | not Orientable |

Every TShape starts Free, Modified and Orientable, with its type in bits 0-3 of
the state word (the `BitLayout` enumeration is the `bit_layout` constants).

`TopoDS_TVertex` has no `EmptyCopy`: it cannot be instantiated alone. The Rust
`TVertex` is a transient struct with the `TopoDS_TVertex` descriptor that holds
the vertex defaults; a derived class (BRep_TVertex when BRep is ported, or a test
fake) embeds it as its parent field, implements `TShape` and supplies
`empty_copy`. `TopoDS_TEdge` has a protected constructor but a public
`EmptyCopy` that returns a plain `TopoDS_TEdge`, so `TEdge` implements `TShape`
and `TEdge::new` is public (Rust has no protected visibility across crates); its
`empty_copy` returns a `TEdge`, which is the native result for a derived edge
class that does not override it.

The derived classes register their parents with `impl_transient!(..., interfaces:
[dyn TShape])`, so `DynamicType`, `IsKind` and the descriptor chain follow
`TopoDS_TSolid -> TopoDS_TShape -> Standard_Transient`.

## 3. Shared mutable state of a TShape

A TShape is shared by every shape that references it, and `Builder::Add`,
`Builder::Remove` and the flag setters change it for all of them. As in
geom-design.md section 3, `TShapeBase` keeps the state behind the TKernel
`SharedState` lock: the child list and the state word are one locked struct, and
every member takes `&self`. A member never calls another shape while it holds the
lock: `Builder::Add` reads the component, then takes the parent's write lock only
to append. The child list is a `Vec<Shape>` (the private
`NCollection_List<TopoDS_Shape> myShapes`): only `TopoDS_Iterator` and
`TopoDS_Builder` read it natively (they are friends), and in Rust only the
iterator (by index) and the builder (append and remove) use it.

The native iterator walks the list nodes. The Rust iterator keeps the TShape
handle and an index. The results are the same while the list does not change;
native behaviour when the current node is removed during an iteration is
undefined (the node is freed), and the Rust iterator then continues at the next
index.

## 4. Shape flags

`Free`, `Modified`, `Checked`, `Orientable`, `Closed`, `Infinite`, `Convex` and
`Locked` are bits 4 to 11 of the TShape state word; they are shared by every
shape of one TShape. `set_modified(true)` also clears `Checked`, as natively.
The native `TopoDS_Shape` flag members dereference the TShape handle without a
check: a null shape crashes. The Rust members return
`Result<_, Failure>` and raise `Standard_NullObject` for a null shape, which is
the exception `TopoDS_Shape::ShapeType` documents. `NbChildren` of a null shape
is 0 natively and in Rust.

## 5. TopoDS_Builder rules

`TopoDS_Builder` is a stateless struct. `make_wire` ... `make_compound` create a
new TShape and set the shape to it with the identity location and the `FORWARD`
orientation (`MakeShape`). `add` first clears the component's `Free` flag
(freezing it even when the call then fails, which prevents self insertion), then
checks, in this order:

1. the parent is free, else `TopoDS_FrozenShape`;
2. the pair is allowed by the native table, else `TopoDS_UnCompatibleShapes`:

| Component | Allowed parents |
| --- | --- |
| COMPOUND, COMPSOLID | COMPOUND |
| SOLID | COMPOUND, COMPSOLID |
| SHELL | COMPOUND, SOLID |
| FACE | COMPOUND, SHELL |
| WIRE | COMPOUND, FACE |
| EDGE | COMPOUND, SOLID, WIRE |
| VERTEX | COMPOUND, SOLID, FACE, EDGE |

The child is stored reversed when the parent is REVERSED, and moved by the
inverse of the parent location, so that iterating with the cumulated orientation
and location gives the component back. `add` sets `Modified`. `remove` raises
`TopoDS_FrozenShape` for a frozen parent, then removes the first child equal
(same TShape, location and orientation) to the component in the parent's
frame, and sets `Modified` only when one was removed. The three exception
classes (`TopoDS_FrozenShape`, `TopoDS_LockedShape`,
`TopoDS_UnCompatibleShapes`, all Standard_DomainError natively) are `Failure`
constructors in `topods::errors`. A null parent or component crashes natively
(null handle dereference); Rust raises `Standard_NullObject`.

TopoDS itself raises no `Standard_ImmutableObject`: that exception is not used by
any TKBRep package at the pinned revision. The `Locked` flag is only stored.

## 6. TopoDS_Iterator and TopExp_Explorer

`TopoDS_Iterator` composes each child orientation with the cumulated orientation
of the shape (`cumOri`) and moves each child by the shape location (`cumLoc`).
`next` past the end is a no-op, as the native list iterator's `++` is. `value`
raises `Standard_NoSuchObject` when there is no current shape.

`TopExp_Explorer` keeps the native depth-first stack of iterators. Its quirks are
kept: `next` after the end restarts the exploration from the root (the stack is
empty again), `current` at the end returns the explored shape, and `next` on an
explorer whose shape is null raises `Standard_NullObject` (a crash natively). Both
types also implement `std::iter::Iterator` (the native `begin`/`end` range).

## 7. Hashing and equality

`IsPartner` compares the TShape handles, `IsSame` also the locations, and
`IsEqual` (and `==`) also the orientations. The native `std::hash<TopoDS_Shape>`
is

```text
hl = hash(Location)            // TopLoc: Murmur chain over datum addresses and powers
hash(shape) = hl == 0 ? MurmurHash64A(&tshape_ptr, 8, 0xA329F1D3A586)
                      : MurmurHash64A(&tshape_ptr, 8, hl)
```

`Shape::hash_code` computes it from the TShape object address (the address the
handle owns, as `TShape().get()`) and `Location::hash_code`. Addresses are
process-specific natively as in Rust, so the formula is also exposed on explicit
parts (`Shape::hash_from_parts(address, location_hash)`); the oracle compares it
bit for bit on synthetic addresses and checks, on each side, that the real hash
equals the formula on the real address. `TopTools_ShapeMapHasher` is the
`HashPolicy` `ShapeMapHasher` (hash above, equality `IsSame`), and the maps of
TopExp are `IndexedMap<Shape, ShapeMapHasher>`,
`IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>` and `Map<Shape, ShapeMapHasher>`.
An indexed map keeps insertion order, so its order is the exploration order in
both implementations. The iteration order of a plain `Map` follows the hash and
therefore the object addresses, natively as in Rust: it is deterministic for a
given set of addresses and is compared through the synthetic hash only.

## 8. TopTools_ShapeSet

`TopTools_ShapeSet` is a class with virtual hooks (`AddGeometry`,
`WriteGeometry`, `ReadGeometry`, `AddShapes`, `Check`...) that BRepTools and
BinTools override. It follows the "protected data of an abstract class" rule of
geom-design.md: the state (the shape map, the location set and the format
number) is `ShapeSetData`, the trait `ShapeSet` reads it through two accessors,
the hooks are provided methods with the native no-op bodies, and the
non-virtual members (`Add`, `Index`, `Dump`, `Write`, `Read`...) are provided
methods written once. `TopToolsShapeSet` is the base class itself. The virtual
`Clear`, `Dump`, `Write` and `Read` are trait methods too, and their base
bodies are the free functions of `toptools::shape_set_io`, so an override can
call the base, as `TopTools_ShapeSet::Write(OS)` is called by derived classes.

The native file includes `BRep_TFace.hxx` but uses nothing from it, so the whole
class is ported without BRep. Text written to `std::cout` by `Read` ("Not a TShape
table"...) goes to the injected `ConsoleOutput` of the GeomTools port.
