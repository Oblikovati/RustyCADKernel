// PackedMap ReSize bucket allocation ordering. Injected throws occur at the Standard call boundary.
#include <NCollection_PackedMap.hxx>
#include <Standard.hxx>
#include <cstdlib>
#include <cstdint>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>
static bool observing = false, fail_request = false;
static std::vector<std::string> events;
void* Standard::AllocateOptimal(const size_t size) {
  if (observing) {
    events.push_back("allocate:" + std::to_string(size));
    if (fail_request) throw 17;
  }
  return std::malloc(size);
}
void Standard::Free(void* address) {
  if (observing) events.push_back(std::string("free:") + (address ? "0" : "1"));
  std::free(address);
}
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> std::vector<T> keys() {
  return {T(0), T(1), T(31), T(32), T(63), T(64), T(3232), T(6464), T(-1)};
}
template<class T> Map<T> initial(unsigned state) {
  Map<T> map(size_t(state == 4 ? 700 : 1));
  if (state == 1) map.ReSize(size_t(0));
  if (state == 2 || state == 3) for (T key : keys<T>()) map.Add(key);
  if (state == 3) for (T key : keys<T>()) map.Remove(key);
  return map;
}
template<class T> std::string snapshot(const Map<T>& map) {
  std::ostringstream out;
  out << map.NbBuckets() << ',' << map.Size() << ',' << map.myNbPackedMapNodes << ',' << (map.myData1 != nullptr);
  out << ',' << (map.myData1 == nullptr || std::uintptr_t(map.myData1) % 16 == 0);
  if (map.myData1 == nullptr) return out.str();
  for (size_t bucket = 0; bucket <= map.NbBuckets(); ++bucket) {
    for (auto node = map.myData1[bucket]; node; node = node->Next())
      out << ',' << bucket << ',' << std::uint64_t(node->Mask()) << ',' << std::uint64_t(node->Data());
  }
  return out.str();
}
template<class T> std::vector<const void*> nodes(const Map<T>& map) {
  std::vector<const void*> result;
  if (map.myData1 == nullptr) return result;
  for (size_t bucket = 0; bucket <= map.NbBuckets(); ++bucket)
    for (auto node = map.myData1[bucket]; node; node = node->Next()) result.push_back(node);
  return result;
}
static bool same_nodes(const std::vector<const void*>& before, const std::vector<const void*>& after) {
  if (before.size() != after.size()) return false;
  for (auto node : before) {
    bool found = false;
    for (auto candidate : after) found |= node == candidate;
    if (!found) return false;
  }
  return true;
}
template<class T> void scenario(const char* name, unsigned state, unsigned index, size_t request, bool fail) {
  auto map = initial<T>(state); const auto before = snapshot(map); const auto addresses = nodes(map);
  auto table = map.myData1; events.clear(); fail_request = fail; observing = true;
  bool thrown = false;
  try { map.ReSize(request); } catch(int value) { thrown = value == 17; }
  observing = false;
  std::cout << name << ':' << state << ':' << index << ':' << fail << ' ' << before << '|';
  for (unsigned i = 0; i < events.size(); ++i) std::cout << (i ? "," : "") << events[i];
  std::cout << '|' << thrown << '|' << snapshot(map) << '|' << same_nodes(addresses, nodes(map))
            << '|' << (table == map.myData1) << '\n';
}
template<class T> void width(const char* name) {
  const size_t requests[] = {0,1000,1009,SIZE_MAX};
  for (unsigned state = 0; state < 5; ++state)
    for (unsigned index = 0; index < 4; ++index) {
      scenario<T>(name, state, index, requests[index], false);
      scenario<T>(name, state, index, requests[index], true);
    }
}
int main() {
  width<int32_t>("i32"); width<uint32_t>("u32"); width<int64_t>("i64");
  width<uint64_t>("u64"); width<int8_t>("i8"); width<bool>("bool");
}
