#include <NCollection_DataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>

struct CopyTrace {
  std::string events;
  bool failKey = false;
};

struct CopyTracked {
  int number;
  char kind;
  CopyTrace* trace;
  CopyTracked(int value, char label, CopyTrace& record)
      : number(value), kind(label), trace(&record) {}
  CopyTracked(const CopyTracked& other)
      : number(other.number), kind(other.kind), trace(other.trace) {
    trace->events += kind;
    if (kind == 'K' && trace->failKey) throw std::runtime_error("injected key copy");
  }
};

struct AssignmentHash {
  std::size_t salt;
  CopyTrace* trace;
  std::size_t operator()(const CopyTracked& key) const {
    trace->events += 'H';
    return static_cast<std::size_t>(key.number) + salt;
  }
  bool operator()(const CopyTracked& first, const CopyTracked& second) const {
    return first.number == second.number;
  }
};
using TestMap = NCollection_DataMap<CopyTracked, CopyTracked, AssignmentHash>;

static void copyAssignment(TestMap& target, const TestMap& source, CopyTrace& trace,
                           const occ::handle<NCollection_BaseAllocator>& allocator) {
  trace.events.clear();
  TestMap& returned = target.Assign(source);
  std::cout << "copy returned=" << (&returned == &target)
            << " events=" << trace.events << " salt=" << target.GetHasher().salt
            << " allocator=" << (target.Allocator() == allocator)
            << " extent=" << target.Extent() << " source=" << source.Extent() << '\n';
}

static void selfAssignment(TestMap& target, CopyTrace& trace) {
  TestMap::Iterator cursor(target);
  const CopyTracked* value = &cursor.Value();
  trace.events.clear();
  trace.failKey = true;
  TestMap& copied = target.Assign(target);
  TestMap& assigned = (target = target);
  TestMap& moved = (target = std::move(target));
  TestMap::Iterator after(target);
  std::cout << "self returned=" << (&copied == &target && &assigned == &target && &moved == &target)
            << " stable=" << (value == &after.Value())
            << " events=" << trace.events << '\n';
  trace.failKey = false;
}

static void failedAssignment(TestMap& target, const TestMap& source, CopyTrace& trace) {
  trace.events.clear();
  trace.failKey = true;
  bool failed = false;
  try { target.Assign(source); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << "failure caught=" << failed << " events=" << trace.events
            << " extent=" << target.Extent() << '\n';
  trace.failKey = false;
}

static void moveAssignment(TestMap& target, TestMap& source, CopyTrace& trace,
                           const occ::handle<NCollection_BaseAllocator>& allocator) {
  target.Bind(CopyTracked(2, 'K', trace), CopyTracked(3, 'V', trace));
  TestMap::Iterator before(source);
  const CopyTracked* value = &before.Value();
  trace.events.clear();
  TestMap& returned = (target = std::move(source));
  TestMap::Iterator after(target);
  std::cout << "move returned=" << (&returned == &target)
            << " stable=" << (value == &after.Value())
            << " salt=" << target.GetHasher().salt << ',' << source.GetHasher().salt
            << " swapped_allocator=" << (source.Allocator() == allocator)
            << " events=" << trace.events << '\n';
}

int main() {
  CopyTrace trace;
  occ::handle<NCollection_BaseAllocator> allocator = new NCollection_IncAllocator(256);
  TestMap source(AssignmentHash{1, &trace}, 101);
  TestMap target(AssignmentHash{17, &trace}, 101, allocator);
  source.Bind(CopyTracked(4, 'K', trace), CopyTracked(9, 'V', trace));
  copyAssignment(target, source, trace, allocator);
  selfAssignment(target, trace);
  failedAssignment(target, source, trace);
  moveAssignment(target, source, trace, allocator);
  TestMap empty(AssignmentHash{9, &trace}, 101);
  TestMap& returned = target.Assign(empty);
  std::cout << "empty returned=" << (&returned == &target)
            << " extent=" << target.Extent() << '\n';
}
