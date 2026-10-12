// Martensite pinned native style routing through actual finite face/edge topology.
#include "martensite_colored_expansion_inputs.hxx"
#include <TopoDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <limits>
#include <cstring>
#include <TopoDS_FrozenShape.hxx>
#include <Prs3d_ShadingAspect.hxx>
class ColoredDispatchProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::dispatchColors;
  using AIS_ColoredShape::AIS_ColoredShape;
  using Groups=DataMapOfDrawerCompd;
};
static int topology_count(const TopoDS_Shape& shape,TopAbs_ShapeEnum kind)
{
  int count=0;
  for(TopExp_Explorer explorer(shape,kind);explorer.More();explorer.Next()) ++count;
  return count;
}
static void group_row(const ColoredDispatchProbe::Groups& groups,
  const occ::handle<AIS_ColoredDrawer>& base,const occ::handle<AIS_ColoredDrawer>& custom)
{
  std::cout<<' '<<groups.Extent();
  for(int index=1;index<=groups.Extent();++index)
  {
    const auto& value=groups.FindFromIndex(index);
    const auto& drawer=groups.FindKey(index);
    std::cout<<' '<<(drawer==base?0:drawer==custom?1:2)<<':'<<value.NbChildren()
      <<':'<<topology_count(value,TopAbs_FACE)<<':'<<topology_count(value,TopAbs_EDGE)
      <<':'<<topology_count(value,TopAbs_VERTEX);
  }
}
static void dispatch_case(int scenario,bool alias)
{
  ColoredExpansionInputs input;
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> base=new AIS_ColoredDrawer(parent);
  const occ::handle<AIS_ColoredDrawer> custom=new AIS_ColoredDrawer(parent);
  ExpansionMap styles;
  const TopoDS_Shape* overrides[]={&input.vertex,&input.edges[0],&input.faces[0],&input.wire,&input.own};
  if(scenario>0&&scenario<=5) styles.Bind(*overrides[scenario-1],custom);
  if(scenario==6) { styles.Bind(input.faces[0],custom);custom->SetHidden(true); }
  if(scenario==7) base->SetHidden(true);
  ColoredDispatchProbe::Groups opened[8],closed;
  auto& closedOutput=alias?opened[TopAbs_FACE]:closed;
  ColoredDispatchProbe probe(input.base);
  bool overridden=probe.dispatchColors(base,input.base,styles,TopAbs_COMPOUND,false,opened,closedOutput);
  std::cout<<"dispatch "<<scenario<<' '<<alias<<' '<<overridden;
  for(int index=0;index<8;++index) group_row(opened[index],base,custom);
  group_row(closed,base,custom);
  std::cout<<'\n';
}
static void closed_case(int scenario)
{
  const TopoDS_Shape box=BRepPrimAPI_MakeBox(2,3,4).Shape();
  if(scenario!=0)BRepMesh_IncrementalMesh mesh(box,0.1,false,0.5);
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> base=new AIS_ColoredDrawer(parent);
  const occ::handle<AIS_ColoredDrawer> custom=new AIS_ColoredDrawer(parent);
  ExpansionMap styles;
  if(scenario>=2)styles.Bind(TopExp_Explorer(box,TopAbs_FACE).Current(),custom);
  if(scenario==2)custom->SetHidden(true);
  if(scenario>=3)
  {
    custom->SetShadingAspect(new Prs3d_ShadingAspect);
    const auto& aspect=custom->ShadingAspect()->Aspect();
    aspect->SetAlphaMode(scenario==3?Graphic3d_AlphaMode_Opaque:scenario==4?Graphic3d_AlphaMode_Blend:scenario==8?Graphic3d_AlphaMode_Mask:Graphic3d_AlphaMode_BlendAuto);
    Graphic3d_MaterialAspect front=aspect->FrontMaterial(),back=aspect->BackMaterial();
    front.SetAlpha(scenario<=4?0.5f:scenario==7?std::numeric_limits<float>::quiet_NaN():1.f);
    back.SetAlpha(0.5f);aspect->SetFrontMaterial(front);aspect->SetBackMaterial(back);
    if(scenario==6)aspect->SetDistinguishOn();else aspect->SetDistinguishOff();
  }
  ColoredDispatchProbe::Groups opened[8],closed;
  ColoredDispatchProbe probe(box);
  bool overridden=probe.dispatchColors(base,box,styles,TopAbs_COMPOUND,false,opened,closed);
  std::cout<<"closed "<<scenario<<' '<<overridden;
  group_row(opened[TopAbs_FACE],base,custom);group_row(closed,base,custom);
  std::cout<<'\n';
}
static void failure_case()
{
  ColoredExpansionInputs input;
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> base=new AIS_ColoredDrawer(parent);
  ColoredDispatchProbe::Groups opened[8],closed;
  ExpansionMap styles;
  TopoDS_Compound frozen;BRep_Builder().MakeCompound(frozen);frozen.Free(false);
  opened[TopAbs_FACE].Add(base,frozen);
  ColoredDispatchProbe probe(input.base);
  try {probe.dispatchColors(base,input.base,styles,TopAbs_COMPOUND,false,opened,closed);}
  catch(const TopoDS_FrozenShape&)
  {
    std::cout<<"failed TopoDS_FrozenShape";
    group_row(opened[TopAbs_FACE],base,base);group_row(opened[TopAbs_WIRE],base,base);std::cout<<'\n';
  }
  frozen.Free(true);
  bool overridden=probe.dispatchColors(base,input.base,styles,TopAbs_COMPOUND,false,opened,closed);
  std::cout<<"repaired "<<overridden;
  group_row(opened[TopAbs_FACE],base,base);group_row(opened[TopAbs_WIRE],base,base);std::cout<<'\n';
}
int main(int argc,char** argv)
{
  if(argc==1)for(int scenario=0;scenario<8;++scenario)
    for(bool alias:{false,true}) dispatch_case(scenario,alias);
  else if(std::strcmp(argv[1],"failure")==0)failure_case();
  else for(int scenario=0;scenario<9;++scenario)closed_case(scenario);
}
