// Martensite pinned OCCT native virtual lifecycle failure order, no geometry substitute.
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Vertex.hxx>
#include <PrsMgr_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
class ColoredDispatchProbe : public AIS_ColoredShape
{
  DEFINE_STANDARD_RTTIEXT(ColoredDispatchProbe, AIS_ColoredShape)
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  occ::handle<PrsMgr_Presentation> presentation;
  int failure = 0;
  void event(const char* label) const
  {
    std::cout << label << ' ' << CustomAspectsMap().Extent() << ' '
      << presentation->MustBeUpdated() << '\n';
  }
  occ::handle<AIS_ColoredDrawer> CustomAspects(const TopoDS_Shape& shape) override
  {
    event("custom-before");
    if (failure == 1) throw Standard_Failure("custom stage=1; expected successful callback");
    auto result = AIS_ColoredShape::CustomAspects(shape);
    event("custom-after");
    if (failure == 2) throw Standard_Failure("custom stage=2; expected successful callback");
    return result;
  }
  void ClearCustomAspects() override
  {
    event("clear-before");
    if (failure == 1) throw Standard_Failure("clear stage=1; expected successful callback");
    AIS_ColoredShape::ClearCustomAspects();
    event("clear-after");
    if (failure == 2) throw Standard_Failure("clear stage=2; expected successful callback");
  }
};
IMPLEMENT_STANDARD_RTTIEXT(ColoredDispatchProbe, AIS_ColoredShape)
class ColoredDispatchPresentation : public PrsMgr_Presentation
{
public:
  ColoredDispatchPresentation(const occ::handle<PrsMgr_PresentationManager>& manager,
    const occ::handle<PrsMgr_PresentableObject>& object)
    : PrsMgr_Presentation(manager, object, 0)
  {
    // Named real native presentation; invalidation is not fabricated by the fixture.
  }
};
static void runtime_views(const occ::handle<ColoredDispatchProbe>& object)
{
  occ::handle<AIS_Shape> parent(object.get());
  occ::handle<AIS_ColoredShape> colored = occ::down_cast<AIS_ColoredShape>(parent);
  const auto& type = object->DynamicType();
  std::cout << "runtime " << type->Name() << ' ' << type->Parent()->Name()
    << ' ' << type->Parent()->Parent()->Name() << ' ' << (colored == object)
    << ' ' << object->GetRefCount() << '\n';
  parent.Nullify(); colored.Nullify();
  std::cout << "runtime-release " << object->GetRefCount() << '\n';
}
static void custom_call(const occ::handle<ColoredDispatchProbe>& object,
  const TopoDS_Shape& key, int failure)
{
  object->failure = failure;
  object->presentation->SetUpdateStatus(false);
  try { object->CustomAspects(key); }
  catch (const Standard_Failure& error) { std::cout << "failure " << error.ExceptionType() << '\n'; }
  object->event("custom-final");
}
static void clear_call(const occ::handle<ColoredDispatchProbe>& object, int failure)
{
  object->failure = failure;
  object->presentation->SetUpdateStatus(false);
  try { object->ClearCustomAspects(); }
  catch (const Standard_Failure& error) { std::cout << "failure " << error.ExceptionType() << '\n'; }
  object->event("clear-final");
}
int main()
{
  TopoDS_Vertex key;
  BRep_Builder().MakeVertex(key, gp_Pnt(1,2,3), 1.e-7);
  occ::handle<ColoredDispatchProbe> object = new ColoredDispatchProbe(key);
  occ::handle<Graphic3d_GraphicDriver> driver = new SvDriver;
  occ::handle<Graphic3d_StructureManager> structures = new Graphic3d_StructureManager(driver);
  occ::handle<PrsMgr_PresentationManager> manager = new PrsMgr_PresentationManager(structures);
  object->presentation = new ColoredDispatchPresentation(manager, object);
  object->Presentations().Append(object->presentation);
  runtime_views(object);
  custom_call(object, key, 1); custom_call(object, key, 2); custom_call(object, key, 0);
  clear_call(object, 1); clear_call(object, 2); clear_call(object, 0);
  object->failure = 0; object->ClearCustomAspects();
  return 0;
}
