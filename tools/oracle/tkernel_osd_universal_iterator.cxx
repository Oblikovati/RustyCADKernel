// OCCT 3d097a: protected owning wrapper, reached through a derived public alias.
#include <OSD_Parallel.hxx>
#include <cassert>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <type_traits>

struct ParallelExposure : OSD_Parallel { using OSD_Parallel::UniversalIterator; };
using Universal = ParallelExposure::UniversalIterator;
using Interface = std::remove_pointer_t<Universal::value_type>;
struct NoncopyPayload {
  int size = 9;
  NoncopyPayload() = default;
  NoncopyPayload(const NoncopyPayload&) = delete;
};
struct Observations { std::string trace; bool failClone = false; bool nullClone = false; };
struct Cursor final : Interface {
  const NoncopyPayload& payload;
  int position;
  Observations& observations;
  Cursor(const NoncopyPayload& p, int i, Observations& o) : payload(p), position(i), observations(o) {}
  ~Cursor() override { observations.trace += 'd'; }
  bool IsEqual(const Interface& other) const override {
    observations.trace += 'e';
    const auto* cursor = dynamic_cast<const Cursor*>(&other);
    return cursor && position == cursor->position;
  }
  void Increment() override {
    observations.trace += 'i';
    assert(position < payload.size);
    ++position;
  }
  Interface* Clone() const override {
    observations.trace += 'c';
    if (observations.failClone) throw std::runtime_error("clone failure");
    return observations.nullClone ? nullptr : new Cursor(payload, position, observations);
  }
};

int position(const Universal& iterator) { return dynamic_cast<const Cursor&>(**iterator).position; }

void emptyAndBorrowedClone() {
  Universal empty;
  Universal explicitNull(nullptr);
  const Universal& constantEmpty = empty;
  assert(*empty == nullptr && *constantEmpty == nullptr && *explicitNull == nullptr);
  NoncopyPayload payload;
  Observations observations;
  auto source = std::make_unique<Universal>(new Cursor(payload, 0, observations));
  Universal copy(*source);
  source.reset();
  ++copy;
  assert(position(copy) == 1 && observations.trace == "cdi");
  std::cout << "empty 1 1 1\nsource_drop_copy " << position(copy) << ' ' << observations.trace << '\n';
}

void comparisonAndIncrement() {
  NoncopyPayload payload;
  Observations observations;
  Universal source(new Cursor(payload, 2, observations));
  Universal copy(source);
  assert(source == copy && !(source != copy));
  auto* identity = &source;
  assert(&(++source) == identity);
  Universal prior = source++;
  assert(position(prior) == 3 && position(source) == 4);
  assert(source != copy && !(source == copy));
  std::cout << "comparison_increment " << position(prior) << ' ' << position(source)
            << ' ' << observations.trace << '\n';
}

void copyAssignmentOrderAndSelf() {
  NoncopyPayload payload;
  Observations observations;
  Universal source(new Cursor(payload, 2, observations));
  Universal target(new Cursor(payload, 5, observations));
  assert(&(target = source) == &target);
  assert(position(target) == 2 && observations.trace == "cd");
  observations.trace.clear();
  target = target;
  assert(position(target) == 2 && observations.trace == "cd");
  std::cout << "assignment_self 2 cd\n";
}

void throwingCloneLeavesTarget() {
  NoncopyPayload payload;
  Observations observations;
  Universal source(new Cursor(payload, 2, observations));
  Universal target(new Cursor(payload, 5, observations));
  observations.failClone = true;
  try { target = source; assert(false); } catch (const std::runtime_error&) {}
  assert(position(target) == 5 && observations.trace == "c");
  try { source++; assert(false); } catch (const std::runtime_error&) {}
  assert(position(source) == 2 && observations.trace == "cc");
  std::cout << "throw_preserves 5 2 cc\n";
}

void nullableCloneBoundary() {
  NoncopyPayload payload;
  Observations observations;
  Universal source(new Cursor(payload, 2, observations));
  Universal target(new Cursor(payload, 5, observations));
  observations.nullClone = true;
  Universal copy(source);
  assert(*copy == nullptr);
  target = source;
  assert(*target == nullptr && observations.trace == "ccd");
  std::cout << "native_nullable_clone 1 1 ccd\n";
}

void nullablePostAndSelf() {
  NoncopyPayload payload;
  Observations observations;
  Universal source(new Cursor(payload, 2, observations));
  observations.nullClone = true;
  Universal prior = source++;
  assert(*prior == nullptr && position(source) == 3);
  source = source;
  assert(*source == nullptr && observations.trace == "cicd");
  std::cout << "native_nullable_post_self 1 3 1 cicd\n";
}

void invalidEmptyOperation(const std::string& mode) {
  Universal empty;
  if (mode == "copy") { Universal copy(empty); }
  else if (mode == "assign") { Universal target; target = empty; }
  else if (mode == "equal") { (void)(empty == empty); }
  else if (mode == "not_equal") { (void)(empty != empty); }
  else if (mode == "pre_increment") { ++empty; }
  else if (mode == "post_increment") { empty++; }
  else throw std::invalid_argument("unknown empty operation " + mode);
}

int main(int argc, char** argv) {
  if (argc > 1) { invalidEmptyOperation(argv[1]); return 0; }
  emptyAndBorrowedClone();
  comparisonAndIncrement();
  copyAssignmentOrderAndSelf();
  throwingCloneLeavesTarget();
  nullableCloneBoundary();
  nullablePostAndSelf();
}
