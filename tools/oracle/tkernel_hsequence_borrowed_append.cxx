// HSequence const-item Append allocates a node before copy and publishes it after success.
#include <NCollection_HSequence.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
static std::string events;
struct CopyAllocator : NCollection_BaseAllocator {
  int live = 0;
  void* Allocate(size_t size) override {
    void* result = NCollection_BaseAllocator::Allocate(size);
    ++live;
    return result;
  }
  void Free(void* address) override {
    --live;
    NCollection_BaseAllocator::Free(address);
  }
};
struct BorrowedItem;
using Sequence = NCollection_Sequence<BorrowedItem>;
using SharedSequence = NCollection_HSequence<BorrowedItem>;
static const Sequence* observed = nullptr;
static const CopyAllocator* allocator = nullptr;
struct BorrowedItem {
  int value;
  bool fail;
  mutable int reads = 0;
  BorrowedItem(int number, bool failing) : value(number), fail(failing) {}
  BorrowedItem(const BorrowedItem& source) : value(source.value), fail(source.fail) {
    ++source.reads;
    events += "c" + std::to_string(value) + "/" + std::to_string(allocator->live)
           + "/" + std::to_string(observed->Size()) + ";";
    if (fail) throw std::runtime_error("copy failure");
  }
  BorrowedItem(BorrowedItem&&) = delete;
  BorrowedItem& operator=(const BorrowedItem&) = delete;
  ~BorrowedItem() { events += "d" + std::to_string(value) + ";"; }
};
static std::string values(const Sequence& sequence)
{
  std::string result;
  for (Sequence::Iterator cursor(sequence); cursor.More(); cursor.Next()) {
    if (!result.empty()) result += ',';
    result += std::to_string(cursor.Value().value);
  }
  return result.empty() ? "-" : result;
}
static void append_case(int prefix, bool fail)
{
  occ::handle<CopyAllocator> arena = new CopyAllocator;
  SharedSequence target{Sequence(arena)};
  for (int i = 0; i < prefix; ++i) target.EmplaceAppend(i, false);
  BorrowedItem donor(9, fail);
  observed = &target; allocator = arena.get(); events.clear(); bool caught = false;
  try { target.Append(donor); } catch (const std::runtime_error&) { caught = true; }
  std::cout << "append" << prefix << '_' << fail << ' ' << values(target) << ' '
            << caught << ' ' << donor.value << ' ' << donor.reads << ' ' << arena->live
            << ' ' << events << '\n';
  events.clear(); target.Clear();
  std::cout << "clear" << prefix << '_' << fail << ' ' << target.Size() << ' '
            << arena->live << ' ' << (events.empty() ? "-" : events) << '\n';
}
static void recovered_case()
{
  occ::handle<CopyAllocator> arena = new CopyAllocator;
  SharedSequence target{Sequence(arena)};
  BorrowedItem bad(9, true), good(10, false);
  observed = &target; allocator = arena.get(); events.clear();
  try { target.Append(bad); } catch (const std::runtime_error&) {}
  target.Append(good);
  std::cout << "recovery " << values(target) << ' ' << bad.reads << ' ' << good.reads
            << ' ' << arena->live << ' ' << events << '\n';
}
int main()
{
  for (int prefix : {0, 2}) for (bool fail : {false, true}) append_case(prefix, fail);
  recovered_case();
}
