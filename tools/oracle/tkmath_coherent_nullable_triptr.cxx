#include <NCollection_BaseAllocator.hxx>
#include <Poly_CoherentTriPtr.hxx>
#include <Poly_CoherentTriangle.hxx>
#include <iostream>
#include <new>

class NullableTriPtrProbe : public Poly_CoherentTriPtr
{
public:
  explicit NullableTriPtrProbe(const Poly_CoherentTriangle* theTriangle)
      : Poly_CoherentTriPtr(theTriangle)
  {
  }
};

static Poly_CoherentTriPtr* create_null_head(
    const occ::handle<NCollection_BaseAllocator>& theAllocator)
{
  void* aMemory = theAllocator->Allocate(sizeof(NullableTriPtrProbe));
  return ::new (aMemory) NullableTriPtrProbe(nullptr);
}

static int count_members(const Poly_CoherentTriPtr& theHead)
{
  int aCount = 0;
  for (Poly_CoherentTriPtr::Iterator anIter(theHead); anIter.More(); anIter.Next()) ++aCount;
  return aCount;
}

static void print_member_nodes(const Poly_CoherentTriPtr& theHead)
{
  Poly_CoherentTriPtr::Iterator anIter(theHead);
  while (anIter.More())
  {
    std::cout << ' ' << anIter.Value().Node(0);
    anIter.Next();
  }
}

int main()
{
  occ::handle<NCollection_BaseAllocator> anAllocator =
      NCollection_BaseAllocator::CommonBaseAllocator();
  Poly_CoherentTriangle aFirst(1, 2, 3), aSecond(4, 5, 6), aThird(7, 8, 9);
  Poly_CoherentTriPtr* aHead = create_null_head(anAllocator);
  aHead->Append(&aSecond, anAllocator);
  aHead->Prepend(&aThird, anAllocator);
  std::cout << "protected_null_constructor " << count_members(*aHead) << '\n';
  aHead->SetTriangle(&aFirst);
  std::cout << "restored_order " << count_members(*aHead);
  print_member_nodes(*aHead);
  std::cout << '\n';
  aHead->SetTriangle(nullptr);
  Poly_CoherentTriPtr* aRemaining = &aHead->Next();
  Poly_CoherentTriPtr::Remove(aHead, anAllocator);
  std::cout << "remove_null_member " << count_members(*aRemaining);
  print_member_nodes(*aRemaining);
  std::cout << '\n';
  Poly_CoherentTriPtr::RemoveList(aRemaining, anAllocator);
  return 0;
}
