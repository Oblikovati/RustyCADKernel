// Pinned OCCT ordered-map removal and BaseMap destruction traversal.
#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Trace {
  std::string events;
  bool active = false;
  bool fail = false;
  void push(const std::string& event) { if (active) events += event + ","; }
};
struct Key {
  int number;
  Trace* trace;
  ~Key() { trace->push("K" + std::to_string(number)); }
};
struct Value {
  int number;
  Trace* trace;
  ~Value() { trace->push("V" + std::to_string(number)); }
};
struct Hasher {
  Trace* trace;
  size_t operator()(const Key& key) const {
    trace->push("H" + std::to_string(key.number));
    if (trace->fail) throw std::runtime_error("injected hash failure");
    return key.number % 2;
  }
  bool operator()(const Key& left, const Key& right) const {
    trace->push("E" + std::to_string(left.number) + ":" + std::to_string(right.number));
    return left.number == right.number;
  }
};
using Set = NCollection_OrderedMap<Key, Hasher>;
using Map = NCollection_OrderedDataMap<Key, Value, Hasher>;
void add(Set& map, int number, Trace& trace) { map.Add(Key{number, &trace}); }
void add(Map& map, int number, Trace& trace) { map.Bind(Key{number, &trace}, Value{number, &trace}); }
bool remove(Set& map, const Key& key) { return map.Remove(key); }
bool remove(Map& map, const Key& key) { return map.UnBind(key); }
template<class T> void populate(T& map, Trace& trace) {
  for (int number : {4, 1, 2, 3, 6, 5}) add(map, number, trace);
  remove(map, Key{1, &trace});
  add(map, 9, trace);
}
template<class T> void removal(const std::string& name, int number, bool empty, bool fail) {
  Trace trace;
  T map(Hasher{&trace}, 101);
  if (!empty) populate(map, trace);
  Key query{number, &trace};
  trace.active = true;
  trace.fail = fail;
  bool result = false;
  try { result = remove(map, query); } catch (const std::runtime_error&) { result = true; }
  trace.active = false;
  trace.fail = false;
  std::cout << name << " " << result << " " << map.Extent() << " " << trace.events << '\n';
}
template<class T> void destruction(const std::string& name, int mode) {
  Trace trace;
  {
    T map(Hasher{&trace}, 101);
    populate(map, trace);
    if (mode == 3) map.ReSize(200);
    trace.active = true;
    if (mode != 2) map.Clear(mode == 1);
  }
  trace.active = false;
  std::cout << name << " " << trace.events << '\n';
}
template<class T> void cases(const std::string& prefix) {
  removal<T>(prefix + "_hit", 3, false, false);
  removal<T>(prefix + "_miss", 7, false, false);
  removal<T>(prefix + "_empty", 3, true, false);
  removal<T>(prefix + "_throw", 3, false, true);
  destruction<T>(prefix + "_clear", 0);
  destruction<T>(prefix + "_release", 1);
  destruction<T>(prefix + "_drop", 2);
  destruction<T>(prefix + "_resized", 3);
}
int main() { cases<Set>("set"); cases<Map>("map"); }
