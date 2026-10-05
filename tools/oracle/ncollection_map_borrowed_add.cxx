#include <NCollection_BaseAllocator.hxx>
#include <NCollection_Map.hxx>
#include <Standard_Type.hxx>
#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <string>

struct KeyTrace {
  std::string events;
  bool failCopy = false;
};

class KeyAllocator final : public NCollection_BaseAllocator {
public:
  DEFINE_STANDARD_RTTI_INLINE(KeyAllocator, NCollection_BaseAllocator)
  explicit KeyAllocator(KeyTrace& record) : trace(record) {}
  void* Allocate(const size_t size) override {
    trace.events += 'A';
    return std::malloc(size);
  }
  void Free(void* pointer) override {
    trace.events += 'F';
    std::free(pointer);
  }
private:
  KeyTrace& trace;
};

struct BorrowedKey {
  std::size_t number;
  KeyTrace* trace;
  BorrowedKey(std::size_t value, KeyTrace& record) : number(value), trace(&record) {}
  BorrowedKey(const BorrowedKey& other) : number(other.number), trace(other.trace) {
    trace->events += 'K';
    if (trace->failCopy) throw std::runtime_error("injected key copy");
  }
  BorrowedKey(BorrowedKey&& other) = default;
};

struct KeyHash {
  std::size_t operator()(const BorrowedKey& key) const {
    key.trace->events += 'H';
    return key.number;
  }
  bool operator()(const BorrowedKey& first, const BorrowedKey& second) const {
    first.trace->events += 'E';
    return first.number == second.number;
  }
};
using TestMap = NCollection_Map<BorrowedKey, KeyHash>;

static void addCases(TestMap& keys, BorrowedKey& key, KeyTrace& trace) {
  trace.events.clear();
  const bool inserted = keys.Add(key);
  std::cout << "add inserted=" << inserted << " events=" << trace.events << '\n';
  trace.events.clear();
  trace.failCopy = true;
  const bool duplicate = keys.Add(key);
  std::cout << "hit inserted=" << duplicate << " events=" << trace.events << '\n';
  trace.failCopy = false;
}

static void addedCases(TestMap& keys, BorrowedKey& key, KeyTrace& trace) {
  keys.Clear(true);
  trace.events.clear();
  const BorrowedKey* stored = &keys.Added(key);
  std::cout << "added separate=" << (stored != &key) << " events=" << trace.events << '\n';
  trace.events.clear();
  trace.failCopy = true;
  const BorrowedKey* returned = &keys.Added(key);
  std::cout << "added_hit stable=" << (stored == returned) << " events=" << trace.events << '\n';
  trace.failCopy = false;
}

static void failureCase(TestMap& keys, BorrowedKey& key, KeyTrace& trace) {
  keys.Clear(true);
  trace.events.clear();
  trace.failCopy = true;
  bool failed = false;
  try { keys.Add(key); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << "failure caught=" << failed << " events=" << trace.events
            << " extent=" << keys.Extent() << '\n';
  trace.failCopy = false;
}

int main() {
  KeyTrace trace;
  occ::handle<NCollection_BaseAllocator> allocator = new KeyAllocator(trace);
  TestMap keys(KeyHash{}, 101, allocator);
  BorrowedKey key(4, trace);
  addCases(keys, key, trace);
  addedCases(keys, key, trace);
  failureCase(keys, key, trace);
  trace.events.clear();
  const bool inserted = keys.Add(std::move(key));
  std::cout << "owned inserted=" << inserted << " events=" << trace.events << '\n';
}
