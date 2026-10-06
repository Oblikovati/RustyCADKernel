// Raw macro operations and explicit object lifetime, without compiler array-cookie assumptions.
#include <NCollection_PackedMap.hxx>
#include <Standard_Failure.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <limits>
#include <new>
#include <cstdlib>
static bool tracing_nodes = false;
static unsigned node_allocations = 0, node_releases = 0;
void* operator new(std::size_t size) {
  void* result = std::malloc(size);
  if (!result) throw std::bad_alloc();
  if (tracing_nodes) ++node_allocations;
  return result;
}
void operator delete(void* pointer) noexcept {
  if (pointer && tracing_nodes) ++node_releases;
  std::free(pointer);
}
void operator delete(void* pointer, std::size_t) noexcept { ::operator delete(pointer); }
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> using Word = typename Map<T>::IndexType;
template<class T> void raw_bytes(const char* name, bool array, unsigned index, size_t bytes) {
  std::cout << name << ':' << (array ? "array" : "scalar") << ':' << index << ' ';
  try {
    auto p = static_cast<unsigned char*>(array ? Map<T>::operator new[](bytes) : Map<T>::operator new(bytes));
    bool zero = true; std::uint64_t sum = 0;
    for (size_t i = 0; i < bytes; ++i) { zero &= p[i] == 0; p[i] = static_cast<unsigned char>(i * 17 + 165); sum += p[i]; }
    std::cout << "ok|" << bytes << '|' << (p != nullptr) << '|' << (std::uintptr_t(p) % 16 == 0) << '|' << zero << '|' << sum;
    if (array) Map<T>::operator delete[](p); else Map<T>::operator delete(p);
  } catch(const Standard_Failure& failure) { std::cout << "error|" << failure.ExceptionType() << '|' << failure.what(); }
  std::cout << '\n';
}
template<class T> void placement(const char* name, unsigned index, size_t bytes) {
  alignas(Map<T>) unsigned char scratch[sizeof(Map<T>) + 1]; std::memset(scratch, 165, sizeof(scratch));
  void* addresses[] = {scratch, scratch + 1, nullptr};
  for (unsigned slot = 0; slot < 3; ++slot) {
    void* result = Map<T>::operator new(bytes, addresses[slot]);
    Map<T>::operator delete(result, addresses[(slot + 1) % 3]);
    bool retained = true; for (unsigned char value : scratch) retained &= value == 165;
    std::cout << name << ":placement:" << index << ':' << slot << ' ' << (result == addresses[slot]) << '|' << retained << '\n';
  }
}
template<class T> void lifetime(const char* name, unsigned count) {
  void* bytes = count == 1 ? Map<T>::operator new(sizeof(Map<T>)) : Map<T>::operator new[](count * sizeof(Map<T>));
  auto maps = static_cast<Map<T>*>(bytes);
  for (unsigned i = 0; i < count; ++i) new (maps + i) Map<T>(size_t(700 + i));
  tracing_nodes = true; node_allocations = node_releases = 0;
  for (unsigned i = 0; i < count; ++i) { maps[i].Add(T(0)); maps[i].Add(T(41)); maps[i].Add(T(-1)); }
  tracing_nodes = false;
  std::cout << name << ":lifetime:" << count << ' ' << node_allocations;
  for (unsigned i = 0; i < count; ++i) {
    std::cout << '|' << maps[i].NbBuckets() << '|' << maps[i].Size();
    for (T value : maps[i]) std::cout << '|' << std::uint64_t(Word<T>(value));
  }
  tracing_nodes = true;
  for (unsigned i = count; i > 0; --i) maps[i - 1].~Map<T>();
  tracing_nodes = false;
  std::cout << '|' << node_releases << '\n';
  if (count == 1) Map<T>::operator delete(bytes); else Map<T>::operator delete[](bytes);
}
template<class T> struct ThrowingMap : Map<T> {
  ThrowingMap() { this->Add(T(41)); throw 17; }
};
template<class T> void failed_placement(const char* name) {
  auto bytes = static_cast<unsigned char*>(Map<T>::operator new(sizeof(ThrowingMap<T>) + 1));
  bytes[sizeof(ThrowingMap<T>)] = 165;
  bool caught = false; tracing_nodes = true; node_allocations = node_releases = 0;
  try { new (bytes) ThrowingMap<T>; } catch(int value) { caught = value == 17; }
  tracing_nodes = false;
  std::cout << name << ":failed_placement " << caught << '|' << node_allocations << '|' << node_releases
            << '|' << (bytes[sizeof(ThrowingMap<T>)] == 165) << '\n';
  Map<T>::operator delete(bytes);
}
template<class T> void width(const char* name) {
  const size_t sizes[] = {0,1,2,7,8,15,16,31,32,33,63,64,65,127,128,257,SIZE_MAX,SIZE_MAX-31};
  for (unsigned i = 0; i < 18; ++i) { raw_bytes<T>(name, false, i, sizes[i]); raw_bytes<T>(name, true, i, sizes[i]); }
  for (unsigned i = 0; i < 3; ++i) placement<T>(name, i, i == 2 ? SIZE_MAX : i);
  void* first = Map<T>::operator new(0); void* second = Map<T>::operator new[](0);
  std::cout << name << ":zero_identity " << (first != second) << '\n';
  Map<T>::operator delete(first); Map<T>::operator delete[](second);
  Map<T>::operator delete(nullptr); Map<T>::operator delete[](nullptr);
  lifetime<T>(name, 1); lifetime<T>(name, 3); failed_placement<T>(name);
}
int main() {
  width<std::int32_t>("i32"); width<std::uint32_t>("u32");
  width<std::int64_t>("i64"); width<std::uint64_t>("u64");
  width<std::int8_t>("i8"); width<bool>("bool");
}
