// OCCT 3d097a exposes private records through protected producers and public aliases.
#include <NCollection_SparseArrayBase.hxx>
#include <OSD_Parallel.hxx>
#include <cassert>
#include <cstddef>
#include <iostream>
#include <new>
#include <type_traits>
#include <utility>

struct SparseExposure : NCollection_SparseArrayBase
{
  using NCollection_SparseArrayBase::getBlock;
};
using InferredBlock = decltype(std::declval<SparseExposure>().getBlock(nullptr));

struct ParallelExposure : OSD_Parallel
{
  using OSD_Parallel::UniversalIterator;
};
using InferredInterface = std::remove_pointer_t<ParallelExposure::UniversalIterator::value_type>;

static_assert(std::is_abstract_v<InferredInterface>);
static_assert(std::has_virtual_destructor_v<InferredInterface>);
static_assert(std::is_same_v<InferredBlock::Cell, unsigned char>);

struct ObservedIterator final : InferredInterface
{
  int position;
  int* destroyed;
  ObservedIterator(int value, int* counter) : position(value), destroyed(counter) {}
  ~ObservedIterator() override { ++*destroyed; }
  bool IsEqual(const InferredInterface& other) const override
  {
    return position == dynamic_cast<const ObservedIterator&>(other).position;
  }
  void Increment() override { ++position; }
  InferredInterface* Clone() const override { return new ObservedIterator(position, destroyed); }
};

static void sparseMemberAccess()
{
  alignas(size_t) unsigned char storage[InferredBlock::Size(9, 4)] = {};
  ::new (storage) size_t(0);
  InferredBlock block(storage, 9, 4);
  *block.Count = 3;
  assert(block.Array == storage + sizeof(size_t));
  assert(block.Bits == storage + sizeof(size_t) + 36);
  assert(InferredBlock::ToArray(storage, 9, 4) == block.Array);
  assert(block.Set(3) == 1 && block.Set(3) == 0 && block.IsSet(3) == 8);
  assert(block.Unset(3) == 8 && block.IsSet(3) == 0);
  std::cout << "sparse " << *block.Count << ' ' << InferredBlock::bitsPerCell() << ' '
            << InferredBlock::Size(9, 4) << '\n';
}

static void interfaceDispatchAndLifetime()
{
  int destroyed = 0;
  InferredInterface* first = new ObservedIterator(3, &destroyed);
  auto copy = first->Clone();
  assert(first->IsEqual(*copy));
  copy->Increment();
  assert(!first->IsEqual(*copy));
  delete first;
  delete copy;
  assert(destroyed == 2);
  std::cout << "interface " << destroyed << '\n';
}

#if defined(REJECT_SPARSE_PRIVATE_NAME)
using Rejected = NCollection_SparseArrayBase::Block;
#elif defined(REJECT_INTERFACE_PRIVATE_NAME)
using Rejected = OSD_Parallel::IteratorInterface;
#elif defined(REJECT_DIRECT_PROTECTED_TYPE)
using Rejected = OSD_Parallel::UniversalIterator;
#elif defined(REJECT_DIRECT_PROTECTED_PRODUCER)
auto rejected(const NCollection_SparseArrayBase& source) { return source.getBlock(nullptr); }
#elif defined(REJECT_INFERRED_ABSTRACT_CONSTRUCTION)
InferredInterface rejected;
#elif defined(REJECT_PRIVATE_ITERATOR_STORAGE)
auto rejected(ParallelExposure::UniversalIterator& value) { return value.myPtr.get(); }
#endif

int main()
{
  sparseMemberAccess();
  interfaceDispatchAndLifetime();
}
