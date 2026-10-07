// OCCT 3d097a: inferred private interface, custom borrowed noncopyable iterator.
#include <OSD_Parallel.hxx>
#include <cassert>
#include <iostream>
#include <memory>
#include <type_traits>

struct ParallelExposure : OSD_Parallel { using OSD_Parallel::UniversalIterator; };
using InferredInterface = std::remove_pointer_t<ParallelExposure::UniversalIterator::value_type>;
static_assert(std::is_abstract_v<InferredInterface>);
static_assert(std::has_virtual_destructor_v<InferredInterface>);

struct NoncopyPayload {
  int length = 3;
  NoncopyPayload() = default;
  NoncopyPayload(const NoncopyPayload&) = delete;
  NoncopyPayload& operator=(const NoncopyPayload&) = delete;
};

struct BorrowedCursor final : InferredInterface {
  const NoncopyPayload& payload;
  int position;
  int& destroyed;
  BorrowedCursor(const NoncopyPayload& p, int pos, int& d) : payload(p), position(pos), destroyed(d) {}
  ~BorrowedCursor() override { ++destroyed; }

#ifndef OMIT_IS_EQUAL
  bool IsEqual(const InferredInterface& other) const override {
    const auto* cursor = dynamic_cast<const BorrowedCursor*>(&other);
    return cursor != nullptr && position == cursor->position;
  }
  #endif
#ifndef OMIT_INCREMENT
  void Increment() override { assert(position < payload.length); ++position; }
#endif

#ifndef OMIT_CLONE
  InferredInterface* Clone() const override { return new BorrowedCursor(payload, position, destroyed); }
#endif
};

struct ForeignCursor final : InferredInterface {
  bool IsEqual(const InferredInterface&) const override { return false; }
  void Increment() override {}
  InferredInterface* Clone() const override { return new ForeignCursor; }
};

int main() {
  NoncopyPayload payload;
  int destroyed = 0;
  std::unique_ptr<InferredInterface> source(new BorrowedCursor(payload, 0, destroyed));
  std::unique_ptr<InferredInterface> copy(source->Clone());
  assert(source->IsEqual(*copy));
  std::cout << "initial_equal 1\n";
  source->Increment();
  assert(!source->IsEqual(*copy));
  std::cout << "independent_positions 1\n";
  source.reset();
  assert(destroyed == 1);
  copy->Increment();
  assert(dynamic_cast<const BorrowedCursor&>(*copy).position == 1);
  assert(!copy->IsEqual(ForeignCursor()));
  std::cout << "source_destroyed_copy_position 1 1\nforeign_equal 0\n";
  copy.reset();
  assert(destroyed == 2);
  std::cout << "virtual_destructions " << destroyed << '\n';
}
