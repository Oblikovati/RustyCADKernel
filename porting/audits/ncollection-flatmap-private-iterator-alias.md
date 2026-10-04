# Flat map iterator alias reachability audit

This audit records a bounded visibility gap in the compiler inventory. It does not change the extractor, captured inventory, or public-API status.

## Finding

`NCollection_FlatMap::iterator_impl` and `NCollection_FlatDataMap::iterator_impl` are private nested templates, but their public `iterator` aliases expose concrete specializations. The public iterator values support directly callable operations such as `More()`, `Key()`, dereference, and increment. The capture currently records those method declarations as `api_visible=false` with `exclusion_reason=private_ancestor`.

The correct future inventory treatment must follow an externally reachable public alias to its specialization and include only the public operations of that specialization. It must not make the private `iterator_impl` name itself reachable or expose all private members of its enclosing map.

## Source and capture identity

OCCT source revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Header | SHA-256 | Relevant declarations |
|---|---|---|
| `NCollection_FlatMap.hxx` | `8453a03cd0622c96cf944cf1d756341cca9dd58824b1e21f5ba56c77046ac3b8` | `private:` at line 83; `iterator_impl` at 140; public operations at 179–194; public `iterator` / `const_iterator` aliases at 234–235; public legacy `Iterator` at 238 |
| `NCollection_FlatDataMap.hxx` | `c358fca1c8a1d3a141bbff7793250173b6eedd475f41f91f1e8d489c2346cf51` | `private:` at line 88; `iterator_impl` at 159; public operations at 198–213; public aliases at 253–254; public legacy `Iterator` at 257 |

Raw declarations artifact inspected: `target/api-proof/occt-template-metadata-merged/occt-declarations.jsonl.gz`, SHA-256 `ecab4391376da98f289f3c81e67f7186736ee8303cf93f1c14f14d608fb03b14`.

Representative exact declaration identities from that artifact:

| Candidate | USR | Capture result |
|---|---|---|
| FlatMap `iterator_impl` class template | `d09e9906a4441319e9016bc6571a8b5c04d767dadcbc8fb8997eb3c991dcee03` | private access |
| FlatMap `iterator` alias | `dd7826f7207125b386a2675564ebdd52abd233834aed953a0c23f7892c4dfd4e` | public |
| FlatMap `const_iterator` alias | `1c16f857621a9c1183e29918427ef6d9effa4ef5d4e2db3d7d544d571da6fad6` | public |
| FlatMap `operator*` | `fc67ee606d43440af823ef12b40c969ea558c68eb1939d5a1eab148b79e37b93` | public member, excluded as private ancestor |
| FlatMap prefix `operator++` | `bc47a5ed48d5064750c8adbb83c53a9cdffe682876bd8ef70f2727759ae5a871` | public member, excluded as private ancestor |
| FlatMap `Key` | `0d86c7917baaab8812ec4b0877229c52fd858f1dc62aaeb404a739a928b683ac` | public member, excluded as private ancestor |
| FlatMap `More` | `72bfe3bc01129b0a99eacf4376ee5205d4e3d3d8c983bbc46c00880f4b9be625` | public member, excluded as private ancestor |
| FlatDataMap `iterator_impl` class template | `b97f8d58a59bd2de4df962266dae70d86ff0cc5f2a94d50e10f11513169833bb` | private access |
| FlatDataMap `iterator` alias | `6f37c67c9a4ef864c4ab3fb958f52e54202c7a04b3007464c46d57c8e736d8e6` | public |
| FlatDataMap `const_iterator` alias | `261c895c42544367b5dd30b9d0773ae2eeb3b7622fb02986ea55c5a43f128827` | public |
| FlatDataMap `operator*` | `3303d8c06ec3591d6318c7c74b68d76918737775e7be85a20c5e56083d3c93b5` | public member, excluded as private ancestor |
| FlatDataMap prefix `operator++` | `0e132d1ddfbc35909083e2bee272d2b9577f11d737c636404f7c638b55721aad` | public member, excluded as private ancestor |
| FlatDataMap `Key` | `b002361bb349cbd730bc622e4e9a54f6f95a734c527dc25f8bd6d83e04e76dea` | public member, excluded as private ancestor |
| FlatDataMap `More` | `7d684294e0b1243fb7dfc5d311e738ccd3a366ae2234359762c3b6c8a53c25d6` | public member, excluded as private ancestor |

## Native compiler controls

The positive fixture `tools/oracle/tkernel_flatmap_private_iterator_alias.cxx` is compiled against the pinned OCCT build headers and `libTKernel.so`. It obtains iterators through public `begin()` APIs, statically checks the public alias types, calls `More`, `Key`, dereference, and prefix increment, and mutates a `FlatDataMap` item through the mutable iterator. GCC 13.3.0 produced:

```text
1/7/7/0;1/3/4/5/0;3/5
```

This matches `tools/oracle/tkernel_flatmap_private_iterator_alias.expected`.

The negative fixture `tools/oracle/tkernel_flatmap_private_iterator_alias.negative.cxx` tries to name `NCollection_FlatMap<int>::iterator_impl<true>` directly. Compilation fails with GCC's diagnostic that the nested class is private. Together, the controls show that the operations are callable through the alias while the helper's own name is inaccessible.

Reproduce from the repository root, substituting the checkout's OCCT build path if needed:

```sh
occt_build=target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_flatmap_private_iterator_alias.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-flatmap-private-iterator
/tmp/tkernel-flatmap-private-iterator
c++ -std=c++17 -I"$occt_build/include/opencascade" -c \
  tools/oracle/tkernel_flatmap_private_iterator_alias.negative.cxx \
  -o /tmp/tkernel-flatmap-negative.o
```

The final command is expected to fail specifically because `iterator_impl` is private. The compiled positive probe used `/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation` and linked that build's `libTKernel.so`.

## Scope and next step

This is evidence for an alias-reachability case, not a general rule for private ancestors. Any inventory fix should preserve alias target specialization (`FlatMap::iterator` is `iterator_impl<true>`; `FlatDataMap::iterator` is `<false>` and `const_iterator` is `<true>`), and prove returned/public alias type reachability without promoting arbitrary private nested declarations. No declaration status is changed by this audit.
