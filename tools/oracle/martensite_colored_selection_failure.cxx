// Zero tangential deflection proves genuine wire partial append, inherited fallback and repaired selection state.
#include <AIS_ColoredShape.hxx>
#include <StdSelect_BRepSelectionTool.hxx>
#include <StdSelect_BRepOwner.hxx>
#include <SelectMgr_Selection.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Elips.hxx>
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <iostream>
class ColoredSelectionFaultProbe:public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::ComputeSelection;
};
static TopoDS_Wire fault_wire()
{
  TopoDS_Wire wire;BRep_Builder builder;builder.MakeWire(wire);
  builder.Add(wire,BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(),3,1),.2,5.4).Shape());
  return wire;
}
static void fault_row(const char* label,int route,int mode,bool failed,const occ::handle<SelectMgr_Selection>& selection,
  const occ::handle<AIS_ColoredShape>& object)
{
  std::cout<<label<<' '<<route<<' '<<mode<<' '<<failed<<' '<<selection->Entities().Size();
  for(const auto& wrapper:selection->Entities())
  {
    const auto& entity=wrapper->BaseSensitive();const auto& owner=entity->OwnerId();
    std::cout<<' '<<entity->DynamicType()->Name()<<' '<<entity->NbSubElements()<<' '<<owner->Priority()
      <<' '<<owner->ComesFromDecomposition()<<' '<<(owner->Selectable()==object);
  }
  std::cout<<'\n';
}
static void invoke_fault_selection(int route,const occ::handle<ColoredSelectionFaultProbe>& object,
  const TopoDS_Wire& wire,const occ::handle<SelectMgr_Selection>& selection,int mode,double deflection)
{
  if(route==0){StdSelect_BRepSelectionTool::ComputeSensitive(wire,new StdSelect_BRepOwner(wire,7),selection,deflection,.5,9,500.,false);return;}
  object->Attributes()->SetMaximalChordialDeviation(deflection);object->ComputeSelection(selection,mode);
}
static void fault_case(int route,int mode)
{
  const TopoDS_Wire wire=fault_wire();
  const occ::handle<ColoredSelectionFaultProbe> object=new ColoredSelectionFaultProbe(wire);
  object->Attributes()->SetAutoTriangulation(false);object->Attributes()->SetTypeOfDeflection(Aspect_TOD_ABSOLUTE);
  if(route==2)object->CustomAspects(BRepBuilderAPI_MakeVertex(gp_Pnt(100,200,300)).Shape())->SetHidden(true);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);
  selection->Add(new Select3D_SensitivePoint(new SelectMgr_EntityOwner(17),gp_Pnt(9,8,7)));
  bool failed=false;
  try{invoke_fault_selection(route,object,wire,selection,mode,1.e-9);}
  catch(const Standard_ConstructionError&){failed=true;}
  fault_row("fault",route,mode,failed,selection,object);
  invoke_fault_selection(route,object,wire,selection,mode,.1);
  fault_row("repaired",route,mode,false,selection,object);
}
int main()
{
  Message::DefaultMessenger()->ChangePrinters().Clear();
  for(int route=0;route<3;++route)for(int mode:{0,3})fault_case(route,mode);
}
