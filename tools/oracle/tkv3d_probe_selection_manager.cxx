// Real manager calls over named callbacks expose ordering and partial failure state.
#include <SelectMgr_SelectionManager.hxx>
#include <StdSelect_BRepSelectionTool.hxx>
class V3ManagedObject : public SelectMgr_SelectableObject {
public:
  int tag,failMode=-999,empty=0;std::vector<int>& events;
  V3ManagedObject(int id,std::vector<int>& trace):tag(id),events(trace){}
  void own(bool value){myHasOwnPresentations=value;}
  void erased(bool value){myDisplayStatus=value?PrsMgr_DisplayStatus_Erased:PrsMgr_DisplayStatus_Displayed;}
  void BoundingBox(Bnd_Box& box) override {box.Update(0.,0.,0.,1.,1.,1.);}
  void UpdateTransformation() override {events.push_back(200+tag);SelectMgr_SelectableObject::UpdateTransformation();}
  void UpdateTransformations(const occ::handle<SelectMgr_Selection>&) override {events.push_back(100+tag);}
protected:
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>&,int) override {}
  void ComputeSelection(const occ::handle<SelectMgr_Selection>& selection,int mode) override {
    events.push_back(tag*10+mode);if(failMode==mode)throw Standard_Failure("fixture compute failure");if(empty)return;
    occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(1);
    occ::handle<Select3D_SensitivePoint> point=new Select3D_SensitivePoint(owner,gp_Pnt(mode,0.,0.));
    point->SetSensitivityFactor(mode+2);selection->Add(point);
  }
};
class V3ManagedSelector : public SelectMgr_ViewerSelector {
public:
  void setState(const occ::handle<SelectMgr_SelectableObject>& object){
    auto set=myMapOfObjectSensitives.Seek(object);ga_int(set?(*set)->Size():-1);ga_flag(set&&(*set)->IsDirty());
  }
};
class V3ManagerFixture {
public:
  std::vector<int> events;std::vector<occ::handle<V3ManagedObject>> objects;
  occ::handle<V3ManagedSelector> selector=new V3ManagedSelector();
  occ::handle<SelectMgr_SelectionManager> manager=new SelectMgr_SelectionManager(selector);
  V3ManagerFixture(int hierarchy){
    for(int i=0;i<3;++i)objects.push_back(new V3ManagedObject(i,events));
    if(hierarchy){objects[0]->AddChild(objects[1]);objects[hierarchy==1?0:1]->AddChild(objects[2]);}
    events.clear();
  }
  void operation(int op,int a,int b,int c){auto object=objects[a];switch(op){
    case 0:manager->Load(object,b);break;case 1:manager->Remove(object);break;
    case 2:manager->Activate(object,b);break;case 3:manager->Deactivate(object,b);break;
    case 4:ga_flag(manager->IsActivated(object,b));break;
    case 5:manager->ClearSelectionStructures(object,b);break;case 6:manager->RestoreSelectionStructures(object,b);break;
    case 7:manager->RecomputeSelection(object,c!=0,b);break;case 8:manager->Update(object,c!=0);break;
    case 9:manager->SetUpdateMode(object,SelectMgr_TypeOfUpdate(c));break;
    case 10:manager->SetUpdateMode(object,b,SelectMgr_TypeOfUpdate(c));break;
    case 11:manager->SetSelectionSensitivity(object,b,c);break;case 12:manager->UpdateSelection(object);break;
    case 13:object->own(c!=0);break;case 14:object->erased(c!=0);break;
    case 15:object->failMode=b;break;case 16:object->empty=c;break;
    case 17:if(auto s=object->Selection(b))s->UpdateBVHStatus(SelectMgr_TypeOfBVHUpdate(c));break;
    case 18:if(auto s=object->Selection(b))s->SetSelectionState(SelectMgr_StateOfSelection(c));break;
    case 19:selector->SetToPrebuildBVH(c!=0,1);break;
    case 20:selector->RemoveSelectableObject(object);break;
  }}
  void state(){
    ga_int(selector->PixelTolerance());ga_flag(selector->ToPrebuildBVH());
    for(auto& object:objects){
      ga_flag(manager->Contains(object));ga_flag(selector->Contains(object));ga_flag(object->HasOwnPresentations());
      ga_int(object->DisplayStatus());selector->setState(object);ga_int(object->Selections().Size());
      for(auto& selection:object->Selections()){
        ga_int(selection->Mode());ga_int(selection->UpdateStatus());ga_int(selection->BVHUpdateStatus());
        ga_int(selection->GetSelectionState());ga_int(selection->Sensitivity());ga_int(selection->Entities().Size());
        for(auto& entity:selection->Entities()){ga_flag(entity->IsActiveForSelection());ga_int(entity->BaseSensitive()->SensitivityFactor());}
      }
    }
    ga_int(events.size());for(int event:events)ga_int(event);events.clear();
  }
};
void v3_selection_manager(std::istream& input){
  GaReader r{input};int hierarchy=r.integer(),steps=r.integer();V3ManagerFixture fixture(hierarchy);fixture.state();
  for(int i=0;i<steps;++i){int op=r.integer(),a=r.integer(),b=r.integer(),c=r.integer();GA_TRY(fixture.operation(op,a,b,c))fixture.state();}ga_end();
}
