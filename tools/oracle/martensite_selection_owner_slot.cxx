// Real geometry callbacks replace an actual const owner-handle argument slot before entity construction.
#define main contour_orientation_fixture_main
#include "martensite_face_contour_orientation.cxx"
#undef main
#include <SelectMgr_Selection.hxx>
#include <SelectBasics_PickResult.hxx>
#include <AIS_ColoredShape.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <StdSelect_BRepOwner.hxx>
class OwnerSlotMutationCurve:public ContourMutationCurve
{
public:
  occ::handle<SelectMgr_EntityOwner>* slot=nullptr;
  const occ::handle<SelectMgr_EntityOwner>* incoming=nullptr;
  mutable bool replace=false;
  const TopoDS_Shape* topology=nullptr;
  mutable bool recordTopology=false;
  occ::handle<SelectMgr_Selection>* selectionSlot=nullptr;
  const occ::handle<SelectMgr_Selection>* selectionIncoming=nullptr;
  mutable bool replaceSelection=false;
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>>* faceSequence=nullptr;
  const occ::handle<SelectMgr_EntityOwner>* faceOwner=nullptr;
  mutable bool appendSequence=false;
  occ::handle<StdSelect_BRepOwner>* brepOwnerSlot=nullptr;
  const occ::handle<StdSelect_BRepOwner>* brepOwnerIncoming=nullptr;
  mutable bool replaceBrepOwner=false;
  gp_Pnt EvalD0(double u) const override
  {
    record_topology_count();
    replace_owner_slot();
    replace_selection_slot();
    append_face_sequence();
    replace_brep_owner_slot();
    return gp_Pnt(u,0,0);
  }
  void record_topology_count() const
  {
    if(!recordTopology)return;
    recordTopology=false;
    std::cout<<"topology-current "<<topology->TShape()->GetRefCount()<<'\n';
  }
  void replace_owner_slot() const
  {
    if(!replace)return;
    replace=false;std::cout<<"slot-before "<<(*slot)->GetRefCount()<<' '<<(*incoming)->GetRefCount()<<'\n';
    *slot=*incoming;
    std::cout<<"slot-replaced "<<(*slot)->GetRefCount()<<' '<<(*incoming)->GetRefCount()<<'\n';
  }
  void replace_selection_slot() const
  {
    if(!replaceSelection)return;
    replaceSelection=false;std::cout<<"selection-before "<<(*selectionSlot)->GetRefCount()<<' '<<(*selectionIncoming)->GetRefCount()<<'\n';
    *selectionSlot=*selectionIncoming;
    std::cout<<"selection-replaced "<<(*selectionSlot)->GetRefCount()<<' '<<(*selectionIncoming)->GetRefCount()<<'\n';
  }
  void append_face_sequence() const
  {
    if(!appendSequence)return;
    appendSequence=false;
    std::cout<<"face-sequence-before "<<faceSequence->Length()<<' '<<(*faceOwner)->GetRefCount()<<'\n';
    faceSequence->Append(new Select3D_SensitivePoint(*faceOwner,gp_Pnt(7,8,9)));
    std::cout<<"face-sequence-appended "<<faceSequence->Length()<<' '<<(*faceOwner)->GetRefCount()<<'\n';
  }
  void replace_brep_owner_slot() const
  {
    if(!replaceBrepOwner)return;
    replaceBrepOwner=false;
    std::cout<<"subshape-owner-before "<<(*brepOwnerSlot)->GetRefCount()<<' '<<(*brepOwnerIncoming)->GetRefCount()<<'\n';
    *brepOwnerSlot=*brepOwnerIncoming;
    std::cout<<"subshape-owner-replaced "<<(*brepOwnerSlot)->GetRefCount()<<' '<<(*brepOwnerIncoming)->GetRefCount()<<'\n';
  }

};
static void owner_slot_case(bool load)
{
  std::cout<<"slot-case "<<load<<'\n';
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<SelectMgr_EntityOwner> previous=new SelectMgr_EntityOwner(19);
  const occ::handle<SelectMgr_EntityOwner> incoming=new SelectMgr_EntityOwner(21);
  occ::handle<SelectMgr_EntityOwner> slot=previous;curve->slot=&slot;curve->incoming=&incoming;curve->replace=true;
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> entity;
  if(load){StdSelect_BRepSelectionTool::ComputeSensitive(edge,slot,selection,.1,.5,9,500.,false);entity=selection->Entities().Value(0)->BaseSensitive();}
  else StdSelect_BRepSelectionTool::GetEdgeSensitive(edge,slot,selection,.1,.5,9,500.,entity);
  std::cout<<"slot-finished "<<entity->OwnerId()->Priority()<<' '<<(entity->OwnerId()==incoming)<<' '<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<selection->Entities().Size()<<'\n';
}
static void selection_slot_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(21);
  const occ::handle<SelectMgr_Selection> previous=new SelectMgr_Selection(0);
  const occ::handle<SelectMgr_Selection> incoming=new SelectMgr_Selection(0);
  occ::handle<SelectMgr_Selection> slot=previous;
  curve->selectionSlot=&slot;curve->selectionIncoming=&incoming;curve->replaceSelection=true;
  StdSelect_BRepSelectionTool::ComputeSensitive(edge,owner,slot,.1,.5,9,500.,false);
  std::cout<<"selection-finished "<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<previous->Entities().Size()<<' '<<incoming->Entities().Size()<<' '<<owner->GetRefCount()<<'\n';
}
static void topology_slot_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  curve->topology=&edge;curve->recordTopology=true;
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(21);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> entity;
  StdSelect_BRepSelectionTool::GetEdgeSensitive(edge,owner,selection,.1,.5,9,500.,entity);
  std::cout<<"topology-finished "<<edge.TShape()->GetRefCount()<<'\n';
}
static TopoDS_Face topology_face_input(const occ::handle<Geom_Curve>& curve)
{
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  BRep_Builder builder;TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,edge);
  NCollection_Array2<gp_Pnt> poles(1,2,1,2);
  poles(1,1)=gp_Pnt(0,0,0);poles(1,2)=gp_Pnt(0,1,0);poles(2,1)=gp_Pnt(1,0,0);poles(2,2)=gp_Pnt(1,1,1);
  TopoDS_Face face;builder.MakeFace(face,new Geom_BezierSurface(poles),1.e-7);builder.Add(face,wire);
  return face;
}
static void topology_face_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Face face=topology_face_input(curve);
  curve->topology=&face;curve->recordTopology=true;
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(21);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  StdSelect_BRepSelectionTool::ComputeSensitive(face,owner,selection,.1,.5,9,500.,false);
  std::cout<<"topology-finished "<<face.TShape()->GetRefCount()<<'\n';
}
static void face_sequence_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Face face=topology_face_input(curve);
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(21);
  const occ::handle<Select3D_SensitiveEntity> seed=new Select3D_SensitivePoint(owner,gp_Pnt(1,2,3));
  NCollection_Sequence<occ::handle<Select3D_SensitiveEntity>> entities;entities.Append(seed);
  const occ::handle<Select3D_SensitiveEntity>& alias=entities.First();
  curve->faceSequence=&entities;curve->faceOwner=&owner;curve->appendSequence=true;
  const bool accepted=StdSelect_BRepSelectionTool::GetSensitiveForFace(face,owner,entities,false,9,500.,true);
  std::cout<<"face-sequence-finished "<<accepted<<' '<<entities.Length()<<' '<<(&alias==&entities.First())<<' '<<(alias==seed)<<' '<<owner->GetRefCount()<<'\n';
}
class ColoredSelectionSlotProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::ComputeSelection;
  using AIS_ColoredShape::computeSubshapeSelection;
};
static void colored_selection_slot_case(bool hidden)
{
  std::cout<<"colored-selection-slot-case "<<hidden<<'\n';
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<ColoredSelectionSlotProbe> object=new ColoredSelectionSlotProbe(edge);
  object->Attributes()->SetupOwnDefaults();object->Attributes()->SetAutoTriangulation(false);
  if(hidden)object->CustomAspects(BRepBuilderAPI_MakeVertex(gp_Pnt(9,8,7)))->SetHidden(true);
  const occ::handle<SelectMgr_Selection> previous=new SelectMgr_Selection(0);
  const occ::handle<SelectMgr_Selection> incoming=new SelectMgr_Selection(0);
  occ::handle<SelectMgr_Selection> slot=previous;
  curve->selectionSlot=&slot;curve->selectionIncoming=&incoming;curve->replaceSelection=true;
  object->ComputeSelection(slot,0);
  const auto entity=incoming->Entities().First()->BaseSensitive();
  std::cout<<"colored-selection-slot-finished "<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<previous->Entities().Size()<<' '<<incoming->Entities().Size()<<' '<<entity->OwnerId()->HasSelectable()<<'\n';
}
static void subshape_owner_slot_case()
{
  const occ::handle<OwnerSlotMutationCurve> curve=new OwnerSlotMutationCurve;
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(curve,0.,1.);
  const occ::handle<ColoredSelectionSlotProbe> object=new ColoredSelectionSlotProbe(edge);
  object->Attributes()->SetAutoTriangulation(false);
  const occ::handle<StdSelect_BRepOwner> previous=new StdSelect_BRepOwner(edge,19,false);
  const occ::handle<StdSelect_BRepOwner> incoming=new StdSelect_BRepOwner(edge,27,false);
  occ::handle<StdSelect_BRepOwner> slot=previous;
  curve->brepOwnerSlot=&slot;curve->brepOwnerIncoming=&incoming;curve->replaceBrepOwner=true;
  const occ::handle<AIS_ColoredDrawer> parent=new AIS_ColoredDrawer(object->Attributes());
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  const NCollection_DataMap<TopoDS_Shape,occ::handle<AIS_ColoredDrawer>,TopTools_ShapeMapHasher> styles;
  object->computeSubshapeSelection(parent,styles,edge,slot,selection,TopAbs_SHAPE,42,.1,.5);
  const auto entity=selection->Entities().First()->BaseSensitive();const auto owner=entity->OwnerId();
  std::cout<<"subshape-owner-finished "<<previous->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<owner->Priority()<<' '<<(owner==incoming)<<' '<<selection->Entities().Size()<<'\n';
}
int main(int argc,char**)
{
  if(argc>5){subshape_owner_slot_case();return 0;}
  if(argc>4){colored_selection_slot_case(false);colored_selection_slot_case(true);return 0;}
  if(argc>3){face_sequence_case();return 0;}
  if(argc>2){topology_face_case();return 0;}
  if(argc>1){topology_slot_case();return 0;}
  owner_slot_case(false);
  owner_slot_case(true);
  selection_slot_case();
}
