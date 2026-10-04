# Builder and ToolsProvider input state

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_builder_argument_state.cxx` and its expected output cover orientation-
insensitive input deduplication, list replacement, Clear, default settings and
configuration persistence. Both SetArguments and SetTools clear their lists but
retain the corresponding identity fence. A previously added shape is therefore
omitted when set again until Clear resets the fence. Rust preserves this behavior.

The Rust classes currently implement only argument/tool and configuration state.
BuilderShape history, Algo execution/progress, PaveFiller and Boolean results are
not implemented by this slice. No StandardTransient inheritance or native RTTI
is claimed: those are absent from the pinned C++ class hierarchy.
All mapped state rows remain partial until their broader object lifecycle and
execution interactions are implemented and tested. These fixtures add no legacy
mapping. Native build links TKBO and its modeling/foundation dependencies.
