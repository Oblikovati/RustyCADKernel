// Pinned OCCT 3d097a0 actual per-subshape appearance state, not a renderer substitute.
#include "martensite_colored_expansion_inputs.hxx"
#include <Prs3d_ShadingAspect.hxx>
#include <Prs3d_LineAspect.hxx>
#include <Prs3d_PointAspect.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Graphic3d_AspectMarker3d.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <cmath>
#include <array>
static long appearance_number(double value)
{
  if (std::isnan(value)) return -999;
  return std::lround(value * 1000.0);
}
static void appearance_color(const Quantity_Color& color)
{
  std::cout << ' ' << appearance_number(color.Red())
            << ' ' << appearance_number(color.Green()) << ' ' << appearance_number(color.Blue());
}
static std::array<occ::handle<Prs3d_LineAspect>,6> appearance_lines(const Prs3d_Drawer& drawer)
{
  return {drawer.LineAspect(), drawer.WireAspect(), drawer.FreeBoundaryAspect(),
          drawer.UnFreeBoundaryAspect(), drawer.SeenLineAspect(), drawer.FaceBoundaryAspect()};
}
static void appearance_drawer(const char* label, const occ::handle<AIS_ColoredDrawer>& drawer)
{
  std::cout << label << ' ' << drawer->HasOwnColor() << ' ' << drawer->HasOwnWidth()
    << ' ' << drawer->HasOwnTransparency() << ' ' << drawer->HasOwnMaterial()
    << ' ' << drawer->HasOwnShadingAspect() << ' ' << drawer->HasOwnLineAspect()
    << ' ' << drawer->HasOwnWireAspect() << ' ' << drawer->HasOwnPointAspect();
  std::cout << ' ' << appearance_number(drawer->Transparency());
  for (const auto& line : appearance_lines(*drawer)) {
    std::cout << ' ' << appearance_number(line->Aspect()->Width());
    appearance_color(line->Aspect()->Color());
  }
  appearance_color(drawer->PointAspect()->Aspect()->Color());
  appearance_color(drawer->ShadingAspect()->Color());
  std::cout << ' ' << appearance_number(drawer->ShadingAspect()->Transparency()) << '\n';
}
static void appearance_parent(const char* label, const occ::handle<AIS_ColoredShape>& object)
{
  std::cout << label << ' ' << object->CustomAspectsMap().Extent() << ' ' << object->HasColor()
    << ' ' << object->HasWidth() << ' ' << object->HasMaterial()
    << ' ' << appearance_number(object->Width()) << ' ' << appearance_number(object->Transparency()) << '\n';
}
static occ::handle<AIS_ColoredShape> appearance_object(const ColoredExpansionInputs& input)
{
  const occ::handle<AIS_ColoredShape> object = new AIS_ColoredShape(input.base);
  object->Attributes()->SetupOwnDefaults();
  return object;
}
