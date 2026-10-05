# ExtendedString UTF-16 C-string search oracle

The pinned inline overloads of `Search(const char16_t*)` and `SearchFromEnd(const char16_t*)` call `extStringLen` before dispatching to the explicit-length implementation. These pointer overloads therefore stop at the first zero unit; the explicit-length overload continues to search the full span. Null and empty inputs produce `-1`.

The fixture uses `BABA` as the receiver and `A\0B` as the needle. Pointer search finds `A` at one-based positions 2 and 4, while explicit-length search for all three units returns `-1`. A `BA` needle returns 1 and 3. Empty and null pointer needles both return `-1`.

Build and run against the pinned modeling-full native library:

```sh
c++ -std=c++17 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_extended_string_c_units_search.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -o /tmp/tkernel_extended_string_c_units_search
/tmp/tkernel_extended_string_c_units_search
```

The rebuilt executable SHA-256 is `4efbdf20466ca8112d7ecdbb2efc95a62f2f1f89717cb6e8b5b8535746dacf70`. The observed output is committed in `tkernel_extended_string_c_units_search.expected`. Header, implementation, library, Rust source, exact C++ declaration IDs, and unique Rustdoc targets are bound in `porting/audits/extended-string-c-units-search.json`.
