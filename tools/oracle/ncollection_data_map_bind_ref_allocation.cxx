#include <NCollection_BaseAllocator.hxx>
#include <NCollection_DataMap.hxx>
#include <Standard_Type.hxx>

#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <string>

struct Trace {
  std::string events;
  bool throwOnKey = false;
};

class TraceAllocator final : public NCollection_BaseAllocator {
public:
  DEFINE_STANDARD_RTTI_INLINE(TraceAllocator, NCollection_BaseAllocator)
  explicit TraceAllocator(Trace& theTrace) : trace(theTrace) {}
  void* Allocate(const size_t theSize) override {
    trace.events += 'A';
    return std::malloc(theSize);
  }
  void Free(void* theAddress) override {
    trace.events += 'F';
    std::free(theAddress);
  }
private:
  Trace& trace;
};
DEFINE_STANDARD_HANDLE(TraceAllocator, NCollection_BaseAllocator)

struct Key {
  int value;
  Trace* trace;
  Key(int theValue, Trace& theTrace) : value(theValue), trace(&theTrace) {}
  Key(const Key& theOther) : value(theOther.value), trace(theOther.trace) {
    trace->events += 'K';
    if (trace->throwOnKey) throw std::runtime_error("key copy");
  }
};

struct Value {
  int value;
  Trace* trace;
  Value(int theValue, Trace& theTrace) : value(theValue), trace(&theTrace) {}
  Value(const Value& theOther) : value(theOther.value), trace(theOther.trace) {
    trace->events += 'V';
  }
  Value(Value&& theOther) noexcept : value(theOther.value), trace(theOther.trace) {}
  Value& operator=(Value&& theOther) noexcept {
    trace->events += 'M';
    value = theOther.value;
    return *this;
  }
  ~Value() { trace->events += 'D'; }
};

struct Hash {
  std::size_t operator()(const Key& theKey) const noexcept {
    theKey.trace->events += 'H';
    return static_cast<std::size_t>(theKey.value);
  }
  bool operator()(const Key& theLeft, const Key& theRight) const noexcept {
    theLeft.trace->events += 'E';
    return theLeft.value == theRight.value;
  }
};

int main() {
  Trace trace;
  occ::handle<TraceAllocator> typedAllocator = new TraceAllocator(trace);
  occ::handle<NCollection_BaseAllocator> allocator = typedAllocator;
  NCollection_DataMap<Key, Value, Hash> map(Hash{}, 101, allocator);
  Key key(4, trace);
  Value value(9, trace);

  trace.events.clear();
  const bool inserted = map.Bind(key, value);
  std::cout << "inserted=" << inserted << " events=" << trace.events
            << " stored=" << map.Find(key).value << '\n';

  trace.events.clear();
  const bool replaced = map.Bind(key, value);
  std::cout << "replaced=" << replaced << " events=" << trace.events
            << " stored=" << map.Find(key).value << '\n';

  map.Clear(true);
  trace.events.clear();
  trace.throwOnKey = true;
  bool failed = false;
  try {
    map.Bind(key, value);
  } catch (const std::runtime_error&) {
    failed = true;
  }
  std::cout << "failed=" << failed << " events=" << trace.events
            << " extent=" << map.Extent() << '\n';
}
