#include <NCollection_BaseAllocator.hxx>
#include <Poly_CoherentNode.hxx>
#include <Poly_CoherentTriangulation.hxx>
#include <Poly_CoherentTriangle.hxx>
#include <Poly_Triangle.hxx>
#include <Poly_Triangulation.hxx>
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

  void Free(void* address) override
  {
    ++frees;
    Standard::Free(address);
  }
};

void print_counts(const CountingAllocator& allocator)
{
  std::cout << allocator.allocations << ' ' << allocator.frees << std::endl;
}

void exercise_mesh(const occ::handle<CountingAllocator>& allocator,
                   const occ::handle<CountingAllocator>& cloneAllocator)
{
  occ::handle<Poly_Triangulation> source = new Poly_Triangulation(4, 1, false);
  source->SetNode(1, gp_Pnt(0., 0., 0.));
  source->SetNode(2, gp_Pnt(1., 0., 0.));
  source->SetNode(3, gp_Pnt(0., 1., 0.));
  source->SetNode(4, gp_Pnt(1., 1., 0.));
  source->SetTriangle(1, Poly_Triangle(1, 2, 3));

  occ::handle<Poly_CoherentTriangulation> mesh =
    new Poly_CoherentTriangulation(source, allocator);
  print_counts(*allocator);
  {
    occ::handle<Poly_CoherentTriangulation> copy = mesh->Clone(cloneAllocator);
    print_counts(*cloneAllocator);
  }
  print_counts(*cloneAllocator);
  Poly_CoherentTriangle* added = mesh->AddTriangle(0, 1, 3);
  print_counts(*allocator);
  mesh->RemoveTriangle(*added);
  print_counts(*allocator);
  {
    Poly_CoherentNode node(gp_XYZ(2., 3., 4.));
    Poly_CoherentTriangle triangle(8, 9, 10);
    node.AddTriangle(triangle, allocator);
    print_counts(*allocator);
    node.Clear(allocator);
    print_counts(*allocator);
  }
}

int main()
{
  occ::handle<CountingAllocator> allocator = new CountingAllocator;
  occ::handle<CountingAllocator> cloneAllocator = new CountingAllocator;
  exercise_mesh(allocator, cloneAllocator);
  print_counts(*allocator);
}
