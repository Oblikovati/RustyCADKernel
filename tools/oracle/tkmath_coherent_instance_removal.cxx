#include <NCollection_BaseAllocator.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <Poly_CoherentTriPtr.hxx>
#include <Poly_CoherentTriangle.hxx>
#include <Standard.hxx>
#include <cstdlib>
#include <iostream>

class CountingBaseAllocator final : public NCollection_BaseAllocator
{
public:
  size_t allocations = 0;
  size_t frees = 0;

  void* Allocate(const size_t size) override
  {
    ++allocations;
    return Standard::Allocate(size);
  }

  void Free(void* address) override
  {
    ++frees;
    Standard::Free(address);
  }
};

class CountingHeapAllocator final : public NCollection_HeapAllocator
{
public:
  size_t allocations = 0;
  size_t frees = 0;

  void* Allocate(const size_t size) override
  {
    ++allocations;
    return std::malloc((size + 3) & ~size_t(3));
  }

  void Free(void* address) override
  {
    ++frees;
    std::free(address);
  }
};

void remove_base_member(std::ostream& out)
{
  occ::handle<CountingBaseAllocator> anOwner = new CountingBaseAllocator;
  occ::handle<CountingBaseAllocator> aRemover = new CountingBaseAllocator;
  Poly_CoherentTriangle              aTriangle(1, 2, 3);
  auto* aMember = new (anOwner) Poly_CoherentTriPtr(aTriangle);
  Poly_CoherentTriPtr::Remove(aMember, aRemover);
  out << "common " << anOwner->allocations << ':' << anOwner->frees << '/'
      << aRemover->allocations << ':' << aRemover->frees;
}

void remove_heap_member(std::ostream& out)
{
  occ::handle<CountingHeapAllocator> anOwner = new CountingHeapAllocator;
  occ::handle<CountingHeapAllocator> aRemover = new CountingHeapAllocator;
  Poly_CoherentTriangle             aTriangle(1, 2, 3);
  auto* aMember = new (anOwner) Poly_CoherentTriPtr(aTriangle);
  Poly_CoherentTriPtr::Remove(aMember, aRemover);
  out << " heap " << anOwner->allocations << ':' << anOwner->frees << '/'
      << aRemover->allocations << ':' << aRemover->frees;
}

int main()
{
  remove_base_member(std::cout);
  remove_heap_member(std::cout);
  std::cout << std::endl;
}
