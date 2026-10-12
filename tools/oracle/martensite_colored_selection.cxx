// Actual ColoredShape ComputeSelection constructs native owners/entities then matches real axis selecting volumes.
#include "martensite_colored_expansion_inputs.hxx"
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <SelectBasics_PickResult.hxx>
#include <iomanip>
class ColoredSelectionProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::ComputeSelection;
};
static TopoDS_Shape selection_input(const ColoredExpansionInputs& input,int scenario)
{
  switch(scenario)
  {
    case 0:return TopoDS_Shape();case 1:{TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);return empty;}
    case 2:return input.vertex;case 3:return input.edges[0];case 4:return input.wire;case 5:return input.faces[0];
    case 6:return BRepPrimAPI_MakeCylinder(2,4).Shape();case 7:return BRepPrimAPI_MakeCone(2,0,4).Shape();
    case 8:return BRepPrimAPI_MakeCone(2,1,4).Shape();case 9:return BRepPrimAPI_MakeSphere(2).Shape();
    case 10:return BRepPrimAPI_MakeBox(2,3,4).Shape();
    case 12:return BRepPrimAPI_MakeTorus(4,1).Shape();
    case 13:return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(),2),0.,M_PI/2.,-2.,2.).Shape();
    case 14:return BRepBuilderAPI_MakeFace(gp_Sphere(gp_Ax3(),2),0.,2.*M_PI,-.4,.4).Shape();
    case 15:return BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3())).Shape();
    case 16:{TopoDS_Shape box=BRepPrimAPI_MakeBox(2,3,4).Shape();BRepMesh_IncrementalMesh mesh(box,.1,false,.5);return box;}
    default:return input.base;
  }
}
static void selection_custom(const occ::handle<AIS_ColoredShape>& object,
  const ColoredExpansionInputs& input,const TopoDS_Shape& shape,int style)
{
  if(style==0)return;
  if(style==1){object->CustomAspects(input.external)->SetHidden(true);return;}
  if(style==2){object->CustomAspects(shape)->SetHidden(true);return;}
  TopExp_Explorer face(shape,TopAbs_FACE);
  if(face.More()){object->CustomAspects(face.Current())->SetHidden(true);return;}
  object->CustomAspects(input.edges[0])->SetHidden(true);
}
static void selection_entity(const occ::handle<Select3D_SensitiveEntity>& entity,const TopoDS_Shape& shape,
  const occ::handle<AIS_ColoredShape>& object)
{
  const auto& owner=entity->OwnerId();
  auto brep=occ::down_cast<StdSelect_BRepOwner>(owner);
  std::cout<<" entity "<<entity->DynamicType()->Name()<<' '<<entity->NbSubElements()<<' '<<owner->Priority()
    <<' '<<owner->ComesFromDecomposition()<<' '<<(owner->Selectable()==object)<<' '<<(!brep.IsNull());
  if(!brep.IsNull())std::cout<<' '<<int(brep->Shape().ShapeType())<<' '<<brep->Shape().IsEqual(shape);
  for(const auto& origin:{gp_Pnt(0,0,20),gp_Pnt(1,2,20),gp_Pnt(50,50,20)})
  {
    SelectMgr_SelectingVolumeManager volume;volume.InitAxisSelectingVolume(gp_Ax1(origin,gp_Dir(0,0,-1)));volume.BuildSelectingVolume();
    SelectBasics_PickResult pick;bool hit=entity->Matches(volume,pick);std::cout<<" pick "<<hit;
    if(hit)std::cout<<' '<<pick.Depth()<<' '<<pick.DistToGeomCenter();
  }
}
static void selection_case(int scenario,int style,int mode)
{
  ColoredExpansionInputs input;const TopoDS_Shape shape=selection_input(input,scenario);
  const occ::handle<ColoredSelectionProbe> object=new ColoredSelectionProbe(shape);
  object->Attributes()->SetAutoTriangulation(false);selection_custom(object,input,shape,style);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);
  const occ::handle<SelectMgr_EntityOwner> sentinel=new SelectMgr_EntityOwner(17);
  selection->Add(new Select3D_SensitivePoint(sentinel,gp_Pnt(9,8,7)));
  bool failed=false;
  try{object->ComputeSelection(selection,mode);}catch(const Standard_Failure&){failed=true;}
  std::cout<<"selection "<<scenario<<' '<<style<<' '<<mode<<' '<<failed<<' '<<selection->Entities().Size();
  for(const auto& wrapper:selection->Entities())selection_entity(wrapper->BaseSensitive(),shape,object);
  std::cout<<'\n';
}
int main(int argc,char**)
{
  std::cout<<std::setprecision(17);
  if(argc>1)
  {
    for(int scenario=12;scenario<17;++scenario)
      for(int style:{0,3})for(int mode:{0,4,17})selection_case(scenario,style,mode);
    return 0;
  }
  for(int scenario=0;scenario<12;++scenario)
    for(int style=0;style<4;++style)
      for(int mode:{0,1,2,3,4,5,6,7,8,17})selection_case(scenario,style,mode);
}
