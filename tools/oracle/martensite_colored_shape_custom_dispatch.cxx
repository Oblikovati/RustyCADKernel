// Reuse the exact named native application subclass and real presentation from the retained lifecycle proof.
#define main martensite_dispatch_fixture_entry
#include "martensite_colored_shape_dispatch.cxx"
#undef main
static void custom_setter_state(const occ::handle<ColoredDispatchProbe>& object, const TopoDS_Shape& key)
{
  occ::handle<AIS_ColoredDrawer> drawer; object->CustomAspectsMap().Find(key,drawer);
  std::cout << "setter-state " << object->CustomAspectsMap().Extent() << ' ' << object->presentation->MustBeUpdated();
  std::cout << ' ' << (!drawer.IsNull() && drawer->HasOwnColor())
    << ' ' << (!drawer.IsNull() && drawer->HasOwnWidth())
    << ' ' << (!drawer.IsNull() && drawer->HasOwnTransparency()) << '\n';
}
static void custom_setter_call(const occ::handle<ColoredDispatchProbe>& object, const TopoDS_Shape& key, int setter, int stage)
{
  object->AIS_ColoredShape::ClearCustomAspects();
  object->failure=stage; object->presentation->SetUpdateStatus(false);
  std::cout << "setter " << setter << ' ' << stage << '\n';
  try {
    if (setter==0) object->SetCustomColor(key,Quantity_Color(Quantity_NOC_RED));
    if (setter==1) object->SetCustomWidth(key,2.5);
    if (setter==2) object->SetCustomTransparency(key,0.4);
  } catch (const Standard_Failure& error) { std::cout << "failure " << error.ExceptionType() << '\n'; }
  custom_setter_state(object,key);
}
int main()
{
  TopoDS_Vertex key; BRep_Builder().MakeVertex(key,gp_Pnt(1,2,3),1.e-7);
  occ::handle<ColoredDispatchProbe> object = new ColoredDispatchProbe(key);
  occ::handle<Graphic3d_GraphicDriver> driver = new SvDriver;
  occ::handle<Graphic3d_StructureManager> structures = new Graphic3d_StructureManager(driver);
  occ::handle<PrsMgr_PresentationManager> manager = new PrsMgr_PresentationManager(structures);
  object->presentation = new ColoredDispatchPresentation(manager,object);
  object->Presentations().Append(object->presentation);
  for (int setter=0;setter<3;++setter) {
    for (int stage : {1,2,0}) custom_setter_call(object,key,setter,stage);
  }
}
