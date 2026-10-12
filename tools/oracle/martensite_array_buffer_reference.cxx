// Martensite actual const fields; OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Graphic3d_ArrayOfPolylines.hxx>
#include <iostream>
static void buffer_reference_trace(int bounds, int edges)
{
  Graphic3d_ArrayOfPolylines array(4,bounds,edges,Graphic3d_ArrayFlags_None);
  const auto& attributes=array.Attributes();
  const auto& indices=array.Indices();
  const auto& boundSlot=array.Bounds();
  std::cout<<"buffer "<<bounds<<' '<<edges<<' '<<attributes->GetRefCount()
    <<' '<<(indices.IsNull()?0:indices->GetRefCount())<<' '<<(boundSlot.IsNull()?0:boundSlot->GetRefCount())
    <<' '<<(&attributes==&array.Attributes())<<' '<<(&indices==&array.Indices())<<' '<<(&boundSlot==&array.Bounds())<<'\n';
}
int main()
{
  buffer_reference_trace(0,0);
  buffer_reference_trace(2,0);
  buffer_reference_trace(2,4);
}
