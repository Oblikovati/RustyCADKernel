#include <NCollection_BaseAllocator.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <Poly_CoherentNode.hxx>
#include <Poly_CoherentTriangle.hxx>
#include <iostream>

int main()
{
  auto aCommon = NCollection_BaseAllocator::CommonBaseAllocator();
  auto aHeap   = NCollection_HeapAllocator::GlobalHeapAllocator();
  Poly_CoherentTriangle aTriangle(0, 1, 2);

  Poly_CoherentNode aHeapNode;
  aHeapNode.AddTriangle(aTriangle, aHeap);
  std::cout << aHeapNode.RemoveTriangle(aTriangle, aCommon) << ' '
            << aHeapNode.IsFreeNode() << '\n';

  Poly_CoherentNode aCommonNode;
  aCommonNode.AddTriangle(aTriangle, aCommon);
  std::cout << aCommonNode.RemoveTriangle(aTriangle, aHeap) << ' '
            << aCommonNode.IsFreeNode() << '\n';
}
