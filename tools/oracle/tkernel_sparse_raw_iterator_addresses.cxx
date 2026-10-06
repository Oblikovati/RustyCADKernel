// OCCT 3d097a retains raw Iterator's last Block even after exhaustion or null rebinding.
#include <NCollection_SparseArrayBase.hxx>
#include <iostream>
#include <stdexcept>

static void discardInteger(void*) {}
static void copyInteger(void* target, void* source)
{
  *static_cast<int*>(target) = *static_cast<int*>(source);
}
static void rejectConstruction(void*, void*) { throw std::runtime_error("rejected"); }

struct RawSparse : NCollection_SparseArrayBase
{
  explicit RawSparse(size_t block = 4) : NCollection_SparseArrayBase(4, block, discardInteger) {}
  void Insert(size_t index, int value) { setValue(index, &value, copyInteger, copyInteger); }
  void Fail(size_t index)
  {
    try { setValue(index, nullptr, rejectConstruction, copyInteger); }
    catch (const std::runtime_error&) {}
  }
  using NCollection_SparseArrayBase::getBlock;
  void* Address(size_t block) { return myData[block]; }
  void* Array(size_t block) { return getBlock(myData[block]).Array; }
  void* Bitmap(size_t block) { return getBlock(myData[block]).Bits; }
};
struct RawIterator : NCollection_SparseArrayBase::Iterator
{
  explicit RawIterator(const RawSparse* source = nullptr) : Iterator(source) {}
  using Iterator::init;
  using Iterator::value;
};

static void occupiedAndExhausted()
{
  RawSparse source; source.Insert(5, 50);
  RawIterator cursor(&source);
  std::cout << "live " << cursor.More() << ' ' << cursor.Index() << ' '
            << *static_cast<int*>(cursor.value()) << '\n';
  cursor.Next();
  std::cout << "exhausted_address " << cursor.More() << ' ' << cursor.Index() << ' '
            << (cursor.value() == source.Bitmap(1)) << '\n';
  cursor.Next();
  std::cout << "exhausted_noop " << cursor.More() << ' ' << cursor.Index() << ' '
            << (cursor.value() == source.Bitmap(1)) << '\n';
  cursor.Restart();
  std::cout << "restart_address " << cursor.More() << ' ' << cursor.Index() << ' '
            << *static_cast<int*>(cursor.value()) << '\n';
}

static void nullAndEmptyRebinding()
{
  RawSparse source, empty; source.Insert(5, 50);
  RawIterator cursor;
  std::cout << "detached " << cursor.More() << '\n';
  cursor.init(&source); cursor.Next(); cursor.init(nullptr);
  cursor.Next(); cursor.Restart();
  std::cout << "detached_again " << cursor.More() << '\n';
  cursor.init(&source);
  std::cout << "rebound " << cursor.More() << ' ' << cursor.Index() << ' '
            << *static_cast<int*>(cursor.value()) << '\n';
  cursor.init(&empty);
  std::cout << "empty_retains_cache " << cursor.More() << ' ' << cursor.Index() << ' '
            << (cursor.value() == source.Array(1)) << '\n';
}

static void allocatedEmptyBlock()
{
  RawSparse failed; failed.Fail(9);
  RawIterator cursor(&failed);
  std::cout << "failed_block_address " << cursor.More() << ' ' << cursor.Index() << ' '
            << (cursor.value() == failed.Bitmap(2)) << '\n';
}
static void crossBlockAddresses()
{
  RawSparse source; source.Insert(5, 50); source.Insert(9, 90);
  RawIterator cursor(&source); cursor.Next();
  std::cout << "cross_block " << cursor.More() << ' ' << cursor.Index() << ' '
            << *static_cast<int*>(cursor.value()) << '\n';
  cursor.Next();
  std::cout << "cross_block_end " << cursor.More() << ' ' << cursor.Index() << ' '
            << (cursor.value() == source.Bitmap(2)) << '\n';
}
static void publiclyReachablePrivateBlock()
{
  RawSparse source; source.Insert(0, 10);
  auto block = source.getBlock(source.Address(0));
  using InferredBlock = decltype(block);
  InferredBlock projected(source.Address(0), 4, 4);
  auto set = block.Set(1); auto present = block.IsSet(1); auto unset = block.Unset(1);
  std::cout << "private_block_reachable " << *block.Count << ' '
            << (block.Array == projected.Array) << ' ' << (block.Bits == projected.Bits) << ' '
            << sizeof(InferredBlock::Cell) << ' ' << InferredBlock::bitsPerCell() << ' '
            << InferredBlock::Size(4, 4) << ' '
            << (InferredBlock::ToArray(source.Address(0), 4, 4) == block.Array) << ' '
            << int(set) << ' ' << int(present) << ' ' << int(unset) << ' ' << int(block.IsSet(1)) << '\n';
}
int main()
{
  occupiedAndExhausted(); nullAndEmptyRebinding(); allocatedEmptyBlock(); crossBlockAddresses(); publiclyReachablePrivateBlock();
}
