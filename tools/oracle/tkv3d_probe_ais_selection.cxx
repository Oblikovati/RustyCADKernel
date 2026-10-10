// Native selected-owner state and callbacks; fixtures never replace production logic.
#include <AIS_Selection.hxx>
#include <SelectMgr_Filter.hxx>
class V3AisSelection : public AIS_Selection {
public:
  using AIS_Selection::appendOwner;
  void mapState(){ga_int(myResultMap.Size());ga_int(myResultMap.NbBuckets());}
};
static int v3_ais_cursor(const AIS_Selection* selection){return selection->More()?selection->Value()->Priority():-1;}
class V3AisOwner : public SelectMgr_EntityOwner {
public:
  bool forced=false;AIS_Selection* selection;mutable std::vector<int> observed;
  V3AisOwner(const occ::handle<SelectMgr_SelectableObject>& object,int tag,AIS_Selection* selected)
    :SelectMgr_EntityOwner(object,tag),selection(selected){}
  bool IsForcedHilight() const override {observed.push_back(v3_ais_cursor(selection));return forced;}
};
class V3AisFilter : public SelectMgr_Filter {
public:
  int mode=0;AIS_Selection* selection;occ::handle<SelectMgr_EntityOwner> extra;mutable std::vector<int> observed;
  V3AisFilter(AIS_Selection* selected,const occ::handle<SelectMgr_EntityOwner>& owner):selection(selected),extra(owner){}
  bool IsOk(const occ::handle<SelectMgr_EntityOwner>& owner) const override {
    observed.push_back(owner->Priority());observed.push_back(selection->Extent());observed.push_back(v3_ais_cursor(selection));
    switch(mode){case 0:return true;case 1:return false;case 2:return owner->Priority()%2==0;
      case 3:throw Standard_Failure("filter mode 3; expected a non-failing mode");
      case 4:selection->AddSelect(extra);return true;}return false;
  }
};
class V3AisScene {
public:
  occ::handle<V3AisSelection> selection=new V3AisSelection();
  occ::handle<SelectMgr_SelectableObject> object=new V3BoxedSelectable(0.,false);
  std::vector<occ::handle<V3AisOwner>> owners;occ::handle<V3AisFilter> filter;
  V3AisScene(){for(int i=0;i<4;++i)owners.push_back(new V3AisOwner(object,i,selection.get()));filter=new V3AisFilter(selection.get(),owners[3]);}
  occ::handle<SelectMgr_EntityOwner> owner(int i){if(i==4)return nullptr;return owners[i];}
  void operation(int op,int a,int b,int c){auto selected=owner(a);switch(op){
    case 0:ga_int(selection->AddSelect(selected));break;
    case 1:ga_int(selection->Select(selected,filter,AIS_SelectionScheme(b),c!=0));break;
    case 2:selection->Clear();break;case 3:selection->ClearAndSelect(selected,filter,c!=0);break;
    case 4:ga_int(selection->appendOwner(selected,filter));break;case 5:selection->Init();break;
    case 6:filter->mode=a;break;case 7:selection->Next();break;case 8:owners[a]->forced=c!=0;break;
    case 9:selected->SetSelectable(c?object:occ::handle<SelectMgr_SelectableObject>());break;
    case 10:selected->SetSelected(c!=0);break;
    case 11:{NCollection_Array1<occ::handle<SelectMgr_EntityOwner>> picked(-3,-2);picked(-3)=selected;picked(-2)=owner(c);
      selection->SelectOwners(picked,AIS_SelectionScheme(b),a%2==0,filter);break;}
  }}
  void state(){
    ga_int(selection->Extent());ga_flag(selection->IsEmpty());ga_int(v3_ais_cursor(selection.get()));selection->mapState();
    for(auto& selected:selection->Objects())ga_int(selected->Priority());
    for(auto& selected:owners){ga_flag(selection->IsSelected(selected));ga_flag(selected->IsSelected());ga_flag(selected->HasSelectable());
      ga_int(selected->observed.size());for(int tag:selected->observed)ga_int(tag);selected->observed.clear();}
    ga_int(filter->observed.size()/3);for(int tag:filter->observed)ga_int(tag);filter->observed.clear();
  }
};
void v3_ais_selection(std::istream& input){
  GaReader r{input};int count=r.integer();V3AisScene scene;scene.state();
  for(int i=0;i<count;++i){int op=r.integer(),a=r.integer(),b=r.integer(),c=r.integer();GA_TRY(scene.operation(op,a,b,c))scene.state();}ga_end();
}
