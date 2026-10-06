// Native primitive HashCode arithmetic and complete allocation events during ReSize.
#include <NCollection_PackedMap.hxx>
#include <Standard.hxx>
#include <array>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <new>
struct Event { char kind; size_t bytes; };
static std::array<Event, 32> trace;
static unsigned event_count = 0;
static bool tracing = false;
static void* old_table = nullptr;
static size_t old_bytes = 0;
static void record(char kind, size_t bytes) {
  if (tracing && event_count < trace.size()) trace[event_count++] = {kind, bytes};
}
void* Standard::AllocateOptimal(const size_t bytes) {
  record('A', bytes);
  return std::malloc(bytes);
}
void Standard::Free(void* pointer) {
  record(pointer == old_table ? 'F' : 'X', pointer ? old_bytes : 0);
  std::free(pointer);
}
void* operator new(std::size_t bytes) {
  record('N', bytes);
  auto pointer = std::malloc(bytes);
  if (!pointer) throw std::bad_alloc();
  return pointer;
}
void operator delete(void* pointer) noexcept { record('D', 0); std::free(pointer); }
void operator delete(void* pointer, std::size_t) noexcept { ::operator delete(pointer); }
void* operator new[](std::size_t bytes) { return ::operator new(bytes); }
void operator delete[](void* pointer) noexcept { ::operator delete(pointer); }
void operator delete[](void* pointer, std::size_t) noexcept { ::operator delete(pointer); }
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> void hashes(const char* name) {
  using Word = typename Map<T>::IndexType;
  const uint64_t masks[] = {0,1,31,32,63,64,UINT32_MAX,UINT64_MAX,uint64_t(1)<<31,uint64_t(1)<<63,0x1234abcdffff1234};
  const size_t moduli[] = {1,2,3,101,1009,SIZE_MAX};
  for (unsigned mask = 0; mask < 11; ++mask) {
    typename Map<T>::PackedMapNode node(Word(masks[mask]), Word(0), nullptr);
    for (unsigned upper = 0; upper < 6; ++upper)
      std::cout << name << ":hash:" << mask << ':' << upper << ' ' << node.HashCode(moduli[upper]) << '\n';
  }
}
template<class T> Map<T> initial(unsigned state) {
  Map<T> map(size_t(state == 3 ? 700 : 1));
  if (state == 1 || state == 2) {
    for (auto bits : {uint64_t(0), uint64_t(1), uint64_t(32), uint64_t(3232), UINT64_MAX}) map.Add(T(bits));
  }
  if (state == 4) map.ReSize(size_t(0));
  return map;
}
template<class T> void rehash(const char* name, unsigned state) {
  auto map = initial<T>(state);
  old_table = map.myData1; old_bytes = (map.NbBuckets() + 1) * sizeof(void*);
  event_count = 0; tracing = true;
  map.ReSize(size_t(state < 2 ? 1000 : 0));
  tracing = false;
  std::cout << name << ":rehash:" << state << ' ' << event_count;
  for (unsigned index = 0; index < event_count; ++index)
    std::cout << '|' << trace[index].kind << ':' << trace[index].bytes;
  std::cout << '|' << map.NbBuckets() << '|' << map.Size();
  for (T key : map) std::cout << '|' << uint64_t(typename Map<T>::IndexType(key));
  std::cout << '\n';
}
template<class T> void width(const char* name) {
  hashes<T>(name);
  for (unsigned state = 0; state < 5; ++state) rehash<T>(name, state);
}
int main() {
  width<int8_t>("i8"); width<uint8_t>("u8"); width<int16_t>("i16"); width<uint16_t>("u16");
  width<int32_t>("i32"); width<uint32_t>("u32"); width<int64_t>("i64"); width<uint64_t>("u64");
  width<size_t>("usize"); width<ptrdiff_t>("isize"); width<bool>("bool");
}
