// Martensite const-handle fixture; pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Prs3d_LineAspect.hxx>
#include <Prs3d_PointAspect.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <iostream>

template<class Wrapper> void aspect_slot_trace(const char* label, Wrapper& wrapper, Wrapper& replacement)
{
  const auto& slot = wrapper.Aspect();
  const auto& alias = wrapper.Aspect();
  auto previous = slot;
  auto incoming = replacement.Aspect();
  std::cout << label << " before " << (&slot == &alias) << ' ' << previous->GetRefCount() << '\n';
  wrapper.SetAspect(incoming);
  std::cout << label << " replaced " << (slot == incoming) << ' ' << previous->GetRefCount() << ' ' << incoming->GetRefCount() << '\n';
  wrapper.SetAspect({});
  std::cout << label << " null " << slot.IsNull() << ' ' << incoming->GetRefCount() << '\n';
}
int main()
{
  Prs3d_LineAspect line(Quantity_NOC_RED, Aspect_TOL_SOLID, 1.0);
  Prs3d_PointAspect point(Aspect_TOM_POINT, Quantity_NOC_RED, 1.0);
  Prs3d_ShadingAspect shading;
  Prs3d_LineAspect nextLine(Quantity_NOC_BLUE, Aspect_TOL_SOLID, 1.0);
  aspect_slot_trace("line", line, nextLine);
  Prs3d_PointAspect nextPoint(Aspect_TOM_POINT, Quantity_NOC_BLUE, 1.0);
  aspect_slot_trace("point", point, nextPoint);
  Prs3d_ShadingAspect nextShading;
  aspect_slot_trace("shading", shading, nextShading);
}
