// Actual protected per-style presentation emits real buffers, hidden flags and failure deflection state.
#include "martensite_colored_expansion_inputs.hxx"
#include <Prs3d_Presentation.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <StdPrs_ShadedShape.hxx>
#include <V3d_Viewer.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <SelectMgr_SelectableObject.hxx>
#include <iomanip>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <vector>
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
class ColoredEmissionProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::addShapesWithCustomProps;
  using AIS_ColoredShape::dispatchColors;
  using Groups=DataMapOfDrawerCompd;
};
static void emission_case(int scenario,int mode,bool fail)
{
  ColoredExpansionInputs input;
  const TopoDS_Shape shape=scenario>=4?BRepPrimAPI_MakeBox(2,3,4).Shape():TopoDS_Shape(input.base);
  if(scenario>=4)BRepMesh_IncrementalMesh mesh(shape,0.1,false,0.5);
  const occ::handle<ColoredEmissionProbe> object=new ColoredEmissionProbe(shape);
  object->Attributes()->SetAutoTriangulation(false);
  object->Attributes()->SetupOwnDefaults();
  const occ::handle<AIS_ColoredDrawer> base=new AIS_ColoredDrawer(object->Attributes());
  const occ::handle<AIS_ColoredDrawer> custom=new AIS_ColoredDrawer(object->Attributes());
  ExpansionMap styles;
  const TopoDS_Shape customFace=scenario>=4?TopExp_Explorer(shape,TopAbs_FACE).Current():input.faces[0];
  if(scenario>=1&&scenario!=4)styles.Bind(customFace,custom);
  if(scenario==2||scenario==8)custom->SetHidden(true);
  if(scenario>=5)
  {
    custom->SetShadingAspect(new Prs3d_ShadingAspect);
    custom->ShadingAspect()->SetColor(Quantity_NOC_BLUE);
  }
  if(scenario>=7)object->Attributes()->SetFaceBoundaryDraw(true);
  if(scenario==3)object->SetInfiniteState(true);
  ColoredEmissionProbe::Groups opened[8],closed;
  object->dispatchColors(base,shape,styles,TopAbs_COMPOUND,false,opened,closed);
  V3DisplayTrace trace;trace.fail=fail?1:-1;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  bool failed=false;
  try {object->addShapesWithCustomProps(prs,opened,closed,mode);}
  catch(const Standard_ProgramError&) {failed=true;}
  std::cout<<"emission "<<scenario<<' '<<mode<<' '<<fail<<' '<<failed<<' '<<prs->Groups().Size()
    <<' '<<base->TypeOfDeflection()<<' '<<base->HasOwnTypeOfDeflection()<<' '<<custom->TypeOfDeflection()
    <<' '<<custom->HasOwnTypeOfDeflection()<<' '<<trace.events.size();
  for(double event:trace.events)std::cout<<' '<<event;
  std::cout<<'\n';
}
int main()
{
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<9;++scenario)
    for(int mode:{0,1,2})for(bool fail:{false,true})emission_case(scenario,mode,fail);
}
