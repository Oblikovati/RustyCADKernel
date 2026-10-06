#include "tkernel_base_map_indexed_proxy_items.hxx"
#include <iostream>

static void construction_rows() {
  IndexedProxyMap map;
  indexed_proxy_calls.clear();
  IndexedProxyRead detached, first(map), copied(first);
  IndexedProxyWrite writable(map, 2);
  IndexedProxyRead converted(writable), assigned;
  const bool copy_identity = &(assigned = copied) == &assigned;
  const bool conversion_identity = &(assigned = writable) == &assigned;
  std::cout << "construction " << detached.Index() << ' ' << detached.More() << ' '
            << first.Index() << ' ' << converted.Index() << ' ' << assigned.Index() << ' '
            << copy_identity << ' ' << conversion_identity << ' '
            << (indexed_proxy_calls.empty() ? "-" : indexed_proxy_calls) << '\n';
}
static void value_rows() {
  IndexedProxyMap map;
  IndexedProxyRead readonly(map);
  IndexedProxyWrite writable(map);
  indexed_proxy_calls.clear();
  int number;
  { auto result = *readonly; number = result.number; }
  std::cout << "const_value " << number << ' ' << indexed_proxy_calls << '\n';
  indexed_proxy_calls.clear();
  { auto result = *writable; result.Set(17); }
  std::cout << "mutable_proxy " << map.values[0] << ' ' << indexed_proxy_calls << '\n';
  indexed_proxy_calls.clear();
  const int const_key = readonly.Key(), mutable_key = writable.Key();
  std::cout << "owned_key " << const_key << ' ' << mutable_key << ' ' << indexed_proxy_calls << '\n';
}
static void offset_rows() {
  IndexedProxyMap map;
  IndexedProxyRead readonly(map);
  IndexedProxyWrite writable(map, 2);
  indexed_proxy_calls.clear();
  int number;
  { auto result = readonly[1]; number = result.number; }
  std::cout << "const_offset " << number << ' ' << readonly.Index() << ' ' << indexed_proxy_calls << '\n';
  indexed_proxy_calls.clear();
  { auto result = writable[-1]; result.Set(19); }
  std::cout << "mutable_offset " << map.values[0] << ' ' << writable.Index() << ' ' << indexed_proxy_calls << '\n';
}
template<class Operation>
static void failed_result(const char* label, Operation operation) {
  indexed_proxy_calls.clear();
  try { operation(); }
  catch (const Standard_OutOfRange& error) {
    std::cout << label << ' ' << indexed_proxy_calls << ' ' << error.what() << '\n';
  }
}
static void failure_rows() {
  IndexedProxyMap map;
  IndexedProxyRead zero(map, 0), end(map, 2);
  IndexedProxyWrite writable(map);
  failed_result("bad_const", [&] { auto result = *zero; });
  failed_result("bad_const_offset", [&] { auto result = end[1]; });
  failed_result("bad_mutable_offset", [&] { auto result = writable[-1]; });
  failed_result("bad_owned_key", [&] { zero.Key(); });
}
static void size_row() {
  IndexedProxyMap map;
  IndexedProxyRead zero(map, 0), past(map, 3);
  indexed_proxy_calls.clear();
  const bool zero_more = zero.More(), past_more = past.More();
  std::cout << "live_size " << zero_more << ' ' << past_more << ' ' << indexed_proxy_calls << '\n';
}
int main() {
  construction_rows();
  value_rows();
  offset_rows();
  failure_rows();
  size_row();
}
