// Actual native WF calls observe const map Value counts from real Graphic3d driver callbacks, without rendering.
#include "martensite_colored_compound_map.hxx"
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Prs3d_Presentation.hxx>
#include <StdPrs_WFShape.hxx>
#include <StdPrs_ShadedShape.hxx>
#include <V3d_Viewer.hxx>
#include <gp_Pln.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <cstring>
#include "tkservice_probe_null_driver.cxx"
struct CompoundReferenceGraphicsTrace
{
  const TopoDS_Compound* compound=nullptr;
  bool fail=false;
  int attempts=0;
  DrawerCompounds* map=nullptr;
  const TopoDS_Compound* replacement=nullptr;
  const TopoDS_Compound* original=nullptr;
};
class CompoundReferenceGroupProbe:public SvGroup
{
public:
  CompoundReferenceGroupProbe(const occ::handle<Graphic3d_Structure>& owner):SvGroup(owner){}
  void AddPrimitiveArray(Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& attributes,const occ::handle<Graphic3d_BoundBuffer>& bounds,bool evaluate) override
  {
    std::cout<<"wf-array "<<kind<<' '<<attributes->NbElements;
    int attribute=0;size_t stride=0;
    const auto* position=attributes->AttributeData(Graphic3d_TOA_POS,attribute,stride);
    for(int index=0;index<attributes->NbElements;++index)
    {
      float point[3];std::memcpy(point,position+index*stride,sizeof(point));
      for(float value:point)std::cout<<' '<<value;
    }
    std::cout<<'\n';
    Graphic3d_Group::AddPrimitiveArray(kind,indices,attributes,bounds,evaluate);
  }
};
static void mutate_compound_reference(CompoundReferenceGraphicsTrace& trace)
{
  trace.map->ChangeFromIndex(1)=*trace.replacement;
  const auto offset=trace.compound->Location().Transformation().TranslationPart();
  std::cout<<"wf-value-assigned "<<trace.original->TShape()->GetRefCount()<<' '<<trace.replacement->TShape()->GetRefCount()
    <<' '<<trace.compound->IsSame(*trace.replacement)<<' '<<offset.X()<<' '<<offset.Y()<<' '<<offset.Z()<<'\n';
}
class CompoundReferenceStructureProbe:public SvCStructure
{
  CompoundReferenceGraphicsTrace& trace;
public:
  CompoundReferenceStructureProbe(const occ::handle<Graphic3d_StructureManager>& owner,CompoundReferenceGraphicsTrace& record):SvCStructure(owner),trace(record){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override
  {
    std::cout<<"wf-group "<<++trace.attempts<<' '<<trace.compound->TShape()->GetRefCount()<<'\n';
    if(trace.fail)throw Standard_ProgramError("group index=1; expected available native group");
    if(trace.replacement!=nullptr)
    {
      if(trace.attempts==1)mutate_compound_reference(trace);
      const occ::handle<Graphic3d_Group> group=new CompoundReferenceGroupProbe(prs);
      myGroups.Append(group);return group;
    }
    return SvCStructure::NewGroup(prs);
  }
};
class CompoundReferenceDriverProbe:public SvDriver
{
  CompoundReferenceGraphicsTrace& trace;
public:
  CompoundReferenceDriverProbe(CompoundReferenceGraphicsTrace& record):trace(record){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& owner) override
  {
    return new CompoundReferenceStructureProbe(owner,trace);
  }
};
static void compound_wireframe_reference_case(bool fail)
{
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;
  drawer->SetupOwnDefaults();drawer->SetAutoTriangulation(false);drawer->SetIsoOnPlane(false);
  const occ::handle<AIS_ColoredDrawer> key=new AIS_ColoredDrawer(drawer);
  TopoDS_Compound compound;BRep_Builder builder;builder.MakeCompound(compound);
  builder.Add(compound,BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),-2,3,-1,4).Shape());
  DrawerCompounds map;map.Add(key,compound);
  CompoundReferenceGraphicsTrace trace;trace.compound=&map.FindFromIndex(1);trace.fail=fail;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new CompoundReferenceDriverProbe(trace));
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  std::cout<<"wf-before "<<fail<<' '<<compound.TShape()->GetRefCount()<<'\n';
  try {StdPrs_WFShape::Add(prs,*trace.compound,drawer);}
  catch(const Standard_ProgramError&) {std::cout<<"wf-failed "<<compound.TShape()->GetRefCount()<<'\n';}
  if(fail){trace.fail=false;StdPrs_WFShape::Add(prs,*trace.compound,drawer);}
  std::cout<<"wf-after "<<prs->Groups().Size()<<' '<<compound.TShape()->GetRefCount()<<'\n';
  map.Clear(false);
  std::cout<<"wf-cleared "<<compound.TShape()->GetRefCount()<<'\n';
}
static void compound_wireframe_live_reference_case(bool shaded)
{
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;
  drawer->SetupOwnDefaults();drawer->SetAutoTriangulation(false);drawer->SetIsoOnPlane(false);
  const occ::handle<AIS_ColoredDrawer> key=new AIS_ColoredDrawer(drawer);
  TopoDS_Compound original,replacement;BRep_Builder builder;
  builder.MakeCompound(original);builder.MakeCompound(replacement);
  builder.Add(original,BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),-2,3,-1,4).Shape());
  builder.Add(replacement,BRepBuilderAPI_MakeVertex(gp_Pnt(1,2,3)).Shape());
  gp_Trsf placement;placement.SetTranslation(gp_Vec(10,20,30));replacement.Location(TopLoc_Location(placement));
  DrawerCompounds map;map.Add(key,original);
  CompoundReferenceGraphicsTrace trace;trace.compound=&map.FindFromIndex(1);
  trace.map=&map;trace.replacement=&replacement;trace.original=&original;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new CompoundReferenceDriverProbe(trace));
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  if(shaded)StdPrs_ShadedShape::Add(prs,*trace.compound,drawer);
  else StdPrs_WFShape::Add(prs,*trace.compound,drawer);
  std::cout<<(shaded?"shade-live-after ":"wf-live-after ")<<prs->Groups().Size()<<' '<<original.TShape()->GetRefCount()<<' '<<replacement.TShape()->GetRefCount()<<'\n';
}
int main(int argc,char**)
{
  if(argc==1)
  {
    compound_wireframe_reference_case(false);
    compound_wireframe_reference_case(true);
  }
  else compound_wireframe_live_reference_case(argc>2);
}
