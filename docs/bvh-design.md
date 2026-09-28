# BVH port design (TKMath/BVH, phase 1)

This note maps OCCT's header-only `BVH` templates to Rust generics. Phase 1
covers the core types: box, ray, tools, sorters, build queue and thread, trees,
object/set/primitive-set abstractions, properties, and the builder base. It also
covers the queue and binned builders. `BVH_PrimitiveSet`'s default constructor
installs a binned builder, and `BVH_QuickSorter_Test` default-constructs a
`BVH_Triangulation`. A later phase adds the linear, spatial-median and sweep-plane
builders, traversal, and the geometry/object sets on top of these types.
`BVH_ShapeSelect`, `BVH_PairDistance` and `BVH_DistanceField` are out of scope.

## Instantiations used upstream

`BVH.cxx` explicitly instantiates boxes, sets, objects, trees, builders, sorters,
primitive sets and triangulations for `float` and `double` with `N` in {2, 3, 4}.
`BVH_QuickSorter` is instantiated only for N = 3 and 4, but its code is valid for
N = 2 as well. `BVH_Transform` exists only as `<T, 4>`. `BVH_DistanceField` uses
N = 3 and 4. The legacy suites use `double` for N = 2, 3 and 4, and `float` for
N = 3. Precision matters: float instantiations compute in single precision
(`float` operations, `std::numeric_limits<float>`), and the port preserves this.
No OCCT code instantiates `BVH_Box<T, 1>`; the N = 1 branches are not ported.

## Scalar and vector mapping

C++ selects the vector type with `BVH::VectorType<T, N>::Type`:
`NCollection_Vec2/3/4<T>`. The Rust port reuses the TKernel types
`rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4}`. It does not add a parallel
vector type.

- `BvhScalar` is a sealed trait for `f32` and `f64`. It supplies
  `numeric_limits` constants, `static_cast` from `double`/`int`, `std::abs`,
  `std::isinf`, and `BVH::IntFloor`. `IntFloor` reproduces the x86-64
  `cvttsd2si`/`cvttss2si` result for NaN and out-of-range values
  (`i32::MIN`). Rust's saturating `as` cast differs.
- `BvhVector` is implemented for `Vec2<T>`, `Vec3<T>` and `Vec4<T>`. It has the
  associated `Scalar` and `DIMENSION`, and it keeps the NCollection operator
  semantics (see below).
- `Dimension<N>: VectorType<T>` and the alias `BvhVecNt<T, N>` spell the C++
  `BVH::VectorType<T, N>::Type` lookup where a `<T, N>` spelling is clearer.

Generic BVH types take one parameter, `V: BvhVector`, not `<T, const N>`. The
reason is that a `<T, N>` struct needs a `where Dimension<N>: VectorType<T>`
clause on every impl and every downstream builder. Rust does not infer that
clause. With `V`, `T` is `V::Scalar` and `N` is `V::DIMENSION`. N-specific C++
specializations become inherent impls on concrete vectors. For example,
`BVH_BaseBox<T, 3>::Transform` becomes `impl<T> BvhBox<Vec3<T>>`.

| C++ | Rust |
| --- | --- |
| `BVH_Box<double, 3>` | `BvhBox<BvhVec3d>` (= `BvhBox<Vec3<f64>>`) |
| `BVH_Ray<float, 3>` | `BvhRay<BvhVec3f>` |
| `BVH_Tools<T, N>::F(...)` | `bvh::tools::f::<V>(...)` |
| `BVH_Tree<T, N, BVH_BinaryTree>` | `BvhTree<V, BvhBinaryTree>` (default arity) |
| `BVH_Tree<T, N, BVH_QuadTree>` | `BvhTree<V, BvhQuadTree>` |
| `BVH_Set<T, N>` / `BVH_Object<T, N>` | traits `BvhSet<V>` / `BvhObject<V>` |
| `BVH_PrimitiveSet<T, N>` | state struct `BvhPrimitiveSet<V>` + trait `BvhPrimitiveSetAccess<V>` |
| `BVH_Sorter<T, N>` | trait `BvhSorter<V>` |
| `BVH_Builder<T, N>` | trait `BvhBuilder<V>` + `BvhBuilderParams` |
| `BVH::Array<T, N>` helpers | `bvh::array::{value, append, size, ...}` over `LinearVector` |

### Operator semantics that affect bits

- `NCollection_Vec3/Vec4<float|double>::operator/(scalar)` multiplies by the
  reciprocal. `Vec2` divides each component. `BVH_Tools::PointTriangleProjection`
  depends on this for its `/ T(3)` and `/ aNorm` results.
- `cwiseMin`/`cwiseMax` return `a < b ? a : b`, but `BVH::BoxMinMax` uses
  `std::min(a, b)`, which returns `b < a ? b : a`. They differ for signed zeros
  and NaNs, so both are kept.
- `BVH::VecComp<T, N>::Get` maps every axis other than 0 (and 1, 2 for wider
  vectors) to the last component. `BvhVector::component` does the same.

## Errors

Tree, array and set accessors go through `NCollection_LinearVector::Value`. The
pinned RelWithDebInfo profile does not define `No_Exception`, so those accessors
raise `Standard_OutOfRange`. The Rust accessors return `Result<_, Failure>`, and
the message has the index and the valid range. `BvhSet` methods, `BvhSorter::perform`
and `BvhBuilder::build` therefore return `Result` as well.

## Trees

`BVH_TreeBase` data (`NodeInfoBuffer`, `MinPointBuffer`, `MaxPointBuffer`, depth)
is public in C++, and the builders write it directly. `BvhTree` exposes the same
buffers and a public `depth_mut`. The arity is a marker type parameter:
`BvhBinaryTree` has `SetOuter/SetInner/Child<K>/ChangeChild<K>`, node addition,
`EstimateSAH` and `CollapseToQuadTree`, and `BvhQuadTree` has `Child<K>`.
`Child<K>` becomes `child(K, node)`, with `K` checked against the arity.

## Sets and primitive sets

C++ `BVH_PrimitiveSet` inherits from both `BVH_Object` and `BVH_Set`, and it
overrides their common `Box()` with a dirty-flag cache. A concrete set
(`BvhTriangulation`) embeds a `BvhPrimitiveSet<V>` state struct. It implements
`BvhSet` (element operations) and `BvhPrimitiveSetAccess` (tree, builder, cache).
The whole-set box of such a set must forward to `cached_box`. This is the one
override that C++ inheritance makes automatic. The tree is owned by value, not
shared through a handle. During `Update` the tree is moved out and back in, so
the builder can take `&mut dyn BvhSet` and `&mut BvhTree` at the same time.

The object dirty flag starts false (`BVH_ObjectTransient()`), so a new primitive
set returns an invalid box and an empty tree until `MarkDirty`. This is preserved.

## Sorting order

- `BVH_QuickSorter` calls `std::sort` (introsort), which is not stable. The order
  of equal centroids is observable after the in-place permutation. `rustycad_tkernel::stl`
  ports the libstdc++ 13 `std::sort` exactly, so ties resolve as they do in the
  GCC oracle. The centroids are evaluated once before the sort. The results are the
  same because `Center` is a const query.
- `BVH_RadixSorter` uses `std::partition`, which is also not stable. It uses the
  libstdc++ bidirectional algorithm, which is ported with the sort. The top six
  bit levels split into two disjoint halves that `OSD_Parallel::For` may sort
  concurrently. The halves are independent, so parallel and serial runs give the
  same order. The port uses the TKernel `osd::Parallel` wrapper.

## Build queue, threads and parallel queue building

`BVH_BuildQueue` keeps its mutex-guarded sequence and atomic counters with the
same memory orderings. `BVH_BuildThread` runs `execute` on a scoped OS thread.
When a build tool fails, the thread leaves the busy count before it returns the
error. Without this, the other workers would wait forever. C++ has no equivalent
path because an escaping exception terminates the process.

In multi-threaded mode, OCCT threads split nodes concurrently through raw
pointers to the set and the tree. Safe Rust cannot share them like that, so
`BvhQueueBuilder` puts the set and the tree behind one mutex. The workers then
process nodes one at a time. The node numbering depends on the schedule, as it
does in OCCT. The single-thread path, which is the default and the one that the
tests and probes compare, is deterministic and bit-identical.
