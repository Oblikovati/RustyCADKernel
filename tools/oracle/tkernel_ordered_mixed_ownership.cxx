// Mixed overload dispatch/copy phases from OCCT 3d097a03. Moves carry values silently.
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Trace {
  std::string events;
  bool active = false;
  bool fail = false;
  int live = 0;
  void push(const std::string& event) { if (active) events += event + ","; }
  void copy(const char* label) {
    push(std::string(label) + (live ? "1" : "0"));
    if (fail) throw std::runtime_error("injected borrowed-argument copy failure");
  }
};
class Arena : public NCollection_IncAllocator {
  Trace* trace;
public:
  explicit Arena(Trace& t) : NCollection_IncAllocator(4096), trace(&t) {}
  void* Allocate(size_t bytes) override { ++trace->live; return NCollection_IncAllocator::Allocate(bytes); }
  void Free(void* pointer) override { --trace->live; NCollection_IncAllocator::Free(pointer); }
};
struct Key {
  int number; Trace* trace;
  Key(int n, Trace& t) : number(n), trace(&t) {}
  Key(const Key& other) : number(other.number), trace(other.trace) { trace->copy("K"); }
  Key(Key&& other) : number(other.number), trace(other.trace) {}
};
struct Value {
  int number; Trace* trace;
  Value(int n, Trace& t) : number(n), trace(&t) {}
  Value(const Value& other) : number(other.number), trace(other.trace) { trace->copy("V"); }
  Value(Value&& other) : number(other.number), trace(other.trace) {}
  Value& operator=(Value&& other) { number = other.number; return *this; }
};
struct Hasher {
  size_t operator()(const Key& key) const { key.trace->push("H"); return key.number; }
  bool operator()(const Key& left, const Key& right) const {
    left.trace->push("E"); return left.number == right.number;
  }
};
using Map = NCollection_OrderedDataMap<Key, Value, Hasher>;
int dispatch(Map& map, Key& key, Value& value, int method) {
  switch (method) {
    case 0: return map.Bind(key, std::move(value));
    case 1: return map.Bound(key, std::move(value))->number;
    case 2: return map.TryBind(key, std::move(value));
    case 3: return map.TryBound(key, std::move(value)).number;
    case 4: return map.Bind(std::move(key), value);
    case 5: return map.Bound(std::move(key), value)->number;
    case 6: return map.TryBind(std::move(key), value);
    default: return map.TryBound(std::move(key), value).number;
  }
}
void scenario(int method, bool hit, bool fail) {
  Trace trace;
  occ::handle<Arena> arena = new Arena(trace);
  Map map(Hasher{}, 101, arena);
  if (hit) map.Bind(Key(1, trace), Value(10, trace));
  Key key(1, trace); Value value(20, trace);
  const Value* before = map.Seek(key);
  trace.active = true; trace.fail = fail;
  int result = -1;
  try { result = dispatch(map, key, value, method); } catch (const std::runtime_error&) {}
  trace.active = false; trace.fail = false;
  const Value* after = map.Seek(key);
  std::cout << method << '_' << hit << '_' << fail << ' ' << result << ' ' << map.Extent()
    << ' ' << (after ? after->number : -1) << ' ' << (!hit || before == after)
    << ' ' << trace.events << '\n';
}
int main() {
  for (int method = 0; method < 8; ++method)
    for (bool hit : {false, true})
      for (bool fail : {false, true}) scenario(method, hit, fail);
}
