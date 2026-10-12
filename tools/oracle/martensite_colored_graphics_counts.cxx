// Actual native driver callbacks observe const-handle upcasts without inventing owning temporaries.
#include "martensite_colored_expansion_inputs.hxx"
#include <Prs3d_Presentation.hxx>
#include <V3d_Viewer.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include "tkservice_probe_null_driver.cxx"
class ColoredCountsProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::Compute;
};
class ColoredCountsGroup:public SvGroup
{
public:
  ColoredCountsGroup(const occ::handle<Graphic3d_Structure>& prs):SvGroup(prs){}
  void aspect_count(const occ::handle<Graphic3d_Aspects>& aspect)
  {
    std::cout<<"count 1 "<<GetRefCount()<<' '<<aspect->GetRefCount()<<" 0 0\n";
  }
  void SetPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override
  {
    aspect_count(aspect);SvGroup::SetPrimitivesAspect(aspect);
  }
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override
  {
    aspect_count(aspect);SvGroup::SetGroupPrimitivesAspect(aspect);
  }
  void AddPrimitiveArray(Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_BoundBuffer>& bounds,bool evaluate) override
  {
    std::cout<<"count 2 "<<GetRefCount()<<' '<<(indices.IsNull()?0:indices->GetRefCount())<<' '<<vertices->GetRefCount()<<' '<<(bounds.IsNull()?0:bounds->GetRefCount())<<'\n';
    Graphic3d_Group::AddPrimitiveArray(kind,indices,vertices,bounds,evaluate);
  }
};
class ColoredCountsStructure:public SvCStructure
{
public:
  ColoredCountsStructure(const occ::handle<Graphic3d_StructureManager>& owner):SvCStructure(owner){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override
  {
    const occ::handle<Graphic3d_Group> group=new ColoredCountsGroup(prs);
    myGroups.Append(group);return group;
  }
};
class ColoredCountsDriver:public SvDriver
{
public:
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& owner) override
  {
    return new ColoredCountsStructure(owner);
  }
};
static void count_case(int mode)
{
  TopoDS_Shape shape=BRepPrimAPI_MakeBox(2,3,4).Shape();BRepMesh_IncrementalMesh mesh(shape,0.1,false,0.5);
  const occ::handle<ColoredCountsProbe> object=new ColoredCountsProbe(shape);
  object->Attributes()->SetupOwnDefaults();object->Attributes()->SetAutoTriangulation(false);
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new ColoredCountsDriver);
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  std::cout<<"mode "<<mode<<'\n';object->Compute(occ::handle<PrsMgr_PresentationManager>(),prs,mode);
}
int main()
{
  count_case(0);
  count_case(1);
}
