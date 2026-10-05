# `TCollection_AsciiString` literal and view surface

The native literal constructor calls `strlen`. The `std::string_view`
constructor delegates to `TCollection_AsciiString(const char*, int)`, whose
pinned implementation also stops at the first NUL. After an embedded NUL has
been stored by another operation, the string-view conversion exposes the full
stored length; `operator<` and `operator>` compare that same full span.

The Rust array conversion is represented by `AsciiString::from_bytes`, which
applies the native C-string prefix rule. `from_view` applies the constructor's
observed prefix rule. The existing `is_less`, `is_greater`, and `as_bytes`
methods preserve complete spans, including embedded NUL bytes.

The native fixture was compiled and run against the modeling-full TKernel
library. OCCT source revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Evidence input | SHA-256 |
| --- | --- |
| `TCollection_AsciiString.hxx` | `5ce11a239588c7bcdfe62652b4152ea0f283207909c125772ed737e833fe6e3c` |
| `TCollection_AsciiString.lxx` | `47aa5de12a3e74380006b3d31d43bbbb41cc23bab8529f7c8cc99fc8f11d8188` |
| `TCollection_AsciiString.cxx` | `e31486afffb8c13e128fee2f903646f47573309ea17392b9ccb21c5f3e3b003a` |
| `libTKernel.so.8.1.0` | `6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee` |
| Native fixture | `2da3f31b074ec99771fe410a2e2e0848c0b96c7335e592455fe8cb9569cc2768` |
| Native expected output | `c9532cafd138f314970fd69dcc3704f5bf7e52ee03572a12e64ba4672cc49a6c` |
| Rust regression | `2219c40d8a1dd70acabaed827b6a32cea45125e0b37ef2cee087d94c5dc6d74a` |

The native executable printed:

```text
literal 2 6162
view_constructor 1 61
converted 3 41007a
less 0
greater 1
```

The `operator<` and `operator>` compiler declarations are the canonical
`std::basic_string_view<char>` overloads. The conversion declaration's
qualified compiler name is `operator basic_string_view`; these exact selectors
replace alias-spelled selectors that did not match compiler inventory rows.
