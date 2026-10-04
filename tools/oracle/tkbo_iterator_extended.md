# Extended Boolean iterator lists

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Ten native scenarios vary selected indices (none, one enlarged vertex, all
vertices, vertices and edges, and a same-domain mapped vertex) and one/two calls.
Each records five typed lists before and after Prepare: 110 output lines.
Replay: `cargo test -p rustycad-tkbo --test bop_ds_iterator extended`.

The port builds the linear BVH from interfering non-solid source shapes, uses
representative boxes for selected same-domain indices, and selects each query
in source-index order. Pair fencing is unordered within one call; stored pair
orientation is retained. Repeated calls append duplicates, and Prepare retains
the extended lists and selection mode, exactly as the native fixture shows.
Non-extended face/face queries continue using the ordinary lists.

Partial: scheduling is serial; allocator/DS-pointer interface, mutation of a
currently active iterator, general topology and complete legacy/API parity are
unproven. The explicit borrowed DS parameter replaces the native retained pointer.
