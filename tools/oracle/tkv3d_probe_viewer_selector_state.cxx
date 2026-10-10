// Protected-state fixtures exercise the unchanged native selector implementation.
#include <SelectMgr_ViewerSelector.hxx>
class V3SelectorState : public SelectMgr_ViewerSelector {
public:
  std::vector<occ::handle<V3BoxedSelectable>> objects;
  std::vector<occ::handle<SelectMgr_Selection>> selections;
  std::vector<occ::handle<SelectMgr_EntityOwner>> owners;
  occ::handle<Select3D_BVHBuilder3d> builder;
  V3SelectorState(){
    builder=EntitySetBuilder();
    for(int i=0;i<3;++i){
      occ::handle<V3BoxedSelectable> object=new V3BoxedSelectable(i*3.,false);
      occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(i);
      occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(object,1);
      selection->Add(new Select3D_SensitivePoint(owner,gp_Pnt()));object->AddSelection(selection,i);
      objects.push_back(object);selections.push_back(selection);owners.push_back(owner);
    }
  }
  int ownerId(const occ::handle<SelectMgr_EntityOwner>& owner){
    if(owner.IsNull())return -1;
    for(size_t i=0;i<owners.size();++i)if(owners[i]==owner)return int(i);
    throw std::runtime_error("unexpected owner identity");
  }
  void seed(int object,int depth,int priority){
    occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(objects[object],priority);
    SelectMgr_SortCriterion criterion;criterion.Entity=new Select3D_SensitivePoint(owner,gp_Pnt(depth,0.,0.));
    criterion.Point=gp_Pnt(depth,1.,2.);criterion.Depth=depth;criterion.SelectionPriority=priority;
    mystored.Add(owner,criterion);myIsSorted=false;owners.push_back(owner);
  }
  void operation(int op,int a,int b,int c){switch(op){
    case 0:SetPixelTolerance(a);break;case 1:SetDepthTolerance(SelectMgr_TypeOfDepthTolerance(a),b);break;
    case 2:SetPickClosest(a!=0);break;case 3:AddSelectableObject(objects[a]);break;
    case 4:RemoveSelectableObject(objects[a]);break;case 5:seed(a,b,c);break;case 6:SortResult();break;
    case 7:Clear();break;case 8:ga_int(ownerId(Picked(a)));break;
    case 9:{const auto& picked=PickedData(a);ga_real(picked.Depth);ga_real(picked.Point.X());break;}
    case 10:ga_flag(RemovePicked(objects[a]));break;case 11:AddSelectionToObject(objects[a],selections[a]);break;
    case 12:RemoveSelectionOfObject(objects[a],selections[a]);break;
    case 13:RebuildObjectsTree(a!=0);break;case 14:RebuildSensitivesTree(objects[a],b!=0);break;
    case 15:ga_text(Status(objects[a]).ToCString());break;
    case 16:{NCollection_List<int> modes;modes.Append(99);ga_flag(Modes(objects[a],modes,SelectMgr_StateOfSelection(b)));ga_int(modes.Size());for(int mode:modes)ga_int(mode);break;}
    case 17:ResetSelectionActivationStatus();break;
    case 18:{NCollection_List<occ::handle<SelectMgr_EntityOwner>> owners;ActiveOwners(owners);std::vector<int> ids;for(auto& owner:owners)ids.push_back(ownerId(owner));std::sort(ids.begin(),ids.end());ga_int(ids.size());for(int id:ids)ga_int(id);break;}
    case 19:SetEntitySetBuilder(a==0?occ::handle<Select3D_BVHBuilder3d>():builder);break;
    case 20:SetToPrebuildBVH(a!=0,b);break;case 21:WaitForBVHBuild();break;case 22:MoveSelectableObject(objects[a]);break;
    case 23:AllowOverlapDetection(a!=0);break;
    case 24:selections[a]->SetSelectionState(SelectMgr_StateOfSelection(b));if(c)selections[a]->Entities().First()->SetActiveForSelection();break;
    case 25:QueueBVHBuild(selections[a]->Entities().First()->BaseSensitive());break;
    case 28:{std::ostringstream stream;DumpJson(stream,a);ga_text(v3_named_dump_pointers(stream.str(),{"SelectableObjects","EntitySetBuilder"}));break;}
    case 26:myTolerances.Add(a);break;case 27:myTolerances.Decrement(a);break;
  }}
  void state(){
    ga_int(CustomPixelTolerance());ga_int(PixelTolerance());ga_real(Sensitivity());ga_flag(ToPickClosest());
    ga_int(DepthToleranceType());ga_real(DepthTolerance());ga_int(NbPicked());ga_flag(myIsSorted);ga_int(myIndexes.Size());
    for(int index:myIndexes)ga_int(index);
    ga_flag(ToPrebuildBVH());ga_flag(EntitySetBuilder().IsNull());
    for(int i=0;i<3;++i){ga_flag(Contains(objects[i]));ga_flag(IsActive(objects[i],i));ga_flag(IsInside(objects[i],i));ga_int(Status(selections[i]));
      auto set=myMapOfObjectSensitives.Seek(objects[i]);ga_int(set?(*set)->Size():-1);ga_flag(set&&(*set)->IsDirty());}
  }
};
void v3_viewer_selector_state(std::istream& input){
  GaReader r{input};int steps=r.integer();V3SelectorState selector;selector.state();
  for(int i=0;i<steps;++i){int op=r.integer(),a=r.integer(),b=r.integer(),c=r.integer();GA_TRY(selector.operation(op,a,b,c))selector.state();}ga_end();
}
