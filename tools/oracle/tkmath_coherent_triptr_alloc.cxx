#include <NCollection_BaseAllocator.hxx>
#include <Poly_CoherentTriPtr.hxx>
#include <Poly_CoherentTriangle.hxx>
#include <Standard.hxx>
#include <iostream>

class CountingAllocator final : public NCollection_BaseAllocator
{
public:
  size_t allocations = 0;
  size_t frees = 0;
  void* Allocate(const size_t size) override
  {
    ++allocations;
    return Standard::Allocate(size);
  }
  void* AllocateOptimal(const size_t size) override
  {
    ++allocations;
    return Standard::AllocateOptimal(size);
  }
  void Free(void* address) override
  {
    ++frees;
    Standard::Free(address);
  }
};

const Poly_CoherentTriPtr* find_member(const Poly_CoherentTriPtr& head, int firstNode)
{
  for (Poly_CoherentTriPtr::Iterator cursor(head); cursor.More(); cursor.Next())
    if (cursor.Value().Node(0) == firstNode) return &cursor.PtrValue();
  return nullptr;
}

void probe_ring(const occ::handle<CountingAllocator>& counter,
                const occ::handle<NCollection_BaseAllocator>& allocator)
{
  Poly_CoherentTriangle first(1, 2, 3), second(4, 5, 6), third(7, 8, 9);
  auto* head = new (allocator) Poly_CoherentTriPtr(first);
  head->Append(&second, allocator);
  const auto* secondMember = &head->Next();
  head->Prepend(&third, allocator);
  const bool stableAfterInsert = find_member(*head, 4) == secondMember;
  Poly_CoherentTriPtr::Iterator cursor(*head);
  cursor.Next();
  cursor.Next();
  Poly_CoherentTriPtr::Remove(const_cast<Poly_CoherentTriPtr*>(&cursor.PtrValue()), allocator);
  const bool stableAfterRemove = find_member(*head, 4) == secondMember;
  std::cout << counter->allocations << ' ' << counter->frees << ' '
            << stableAfterInsert << ' ' << stableAfterRemove << ' '
            << secondMember->GetTriangle().Node(0) << '\n';
  Poly_CoherentTriPtr::RemoveList(head, allocator);
  std::cout << counter->allocations << ' ' << counter->frees << '\n';
}

int main()
{
  occ::handle<CountingAllocator> counter = new CountingAllocator;
  occ::handle<NCollection_BaseAllocator> allocator = counter;
  probe_ring(counter, allocator);
}
