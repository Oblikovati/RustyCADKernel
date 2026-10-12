// OCCT 3d097a0 output assignment retains the old pointer through its owner's destructor.
#include <StdSelect_BRepSelectionTool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Edge.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <Select3D_SensitiveSegment.hxx>
#include <iostream>
class RetiringOutputOwner:public SelectMgr_EntityOwner
{
  const occ::handle<Select3D_SensitiveEntity>& output;
  const occ::handle<SelectMgr_EntityOwner>& incoming;
public:
  const Select3D_SensitiveEntity* expected=nullptr;
  RetiringOutputOwner(const occ::handle<Select3D_SensitiveEntity>& slot,
    const occ::handle<SelectMgr_EntityOwner>& next):SelectMgr_EntityOwner(5),output(slot),incoming(next){}
  ~RetiringOutputOwner() override
  {
    std::cout<<"output-retiring "<<(output.get()==expected)<<' '<<output->GetRefCount()<<' '<<incoming->GetRefCount()<<'\n';
  }
};
static void output_replacement_case()
{
  const TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,0,0));
  const occ::handle<SelectMgr_EntityOwner> incoming=new SelectMgr_EntityOwner(21);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> output;
  occ::handle<RetiringOutputOwner> previous=new RetiringOutputOwner(output,incoming);
  output=new Select3D_SensitivePoint(previous,gp_Pnt(9,8,7));
  previous->expected=output.get();previous.Nullify();
  StdSelect_BRepSelectionTool::GetEdgeSensitive(edge,incoming,selection,.1,.5,9,500.,output);
  std::cout<<"output-finished "<<output->IsKind(STANDARD_TYPE(Select3D_SensitiveSegment))<<' '<<output->GetRefCount()<<' '<<incoming->GetRefCount()<<' '<<(output->OwnerId()==incoming)<<'\n';
}
int main()
{
  output_replacement_case();
  return 0;
}
