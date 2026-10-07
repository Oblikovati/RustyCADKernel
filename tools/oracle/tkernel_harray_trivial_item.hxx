#pragma once
#include <type_traits>
// Shared explicit trivial policy, with no constructor/assignment hook to dispatch.
struct TrivialItem { int number; };
static_assert(std::is_trivially_copyable<TrivialItem>::value);
