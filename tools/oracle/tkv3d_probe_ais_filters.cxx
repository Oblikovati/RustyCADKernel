// The same callback is evaluated through all native composition classes.
#include <SelectMgr_AndFilter.hxx>
#include <SelectMgr_OrFilter.hxx>
#include <SelectMgr_AndOrFilter.hxx>
void v3_ais_filters(std::istream& input){
  GaReader r{input};int kind=r.integer(),disabled=r.integer(),count=r.integer();
  occ::handle<AIS_Selection> selection=new AIS_Selection();occ::handle<SelectMgr_SelectableObject> object=new V3BoxedSelectable(0.,false);
  occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(object,0);std::vector<occ::handle<V3AisFilter>> callbacks;
  occ::handle<SelectMgr_AndFilter> andFilter=new SelectMgr_AndFilter();occ::handle<SelectMgr_OrFilter> orFilter=new SelectMgr_OrFilter();
  occ::handle<SelectMgr_AndOrFilter> combined=new SelectMgr_AndOrFilter(kind==2?SelectMgr_FilterType_AND:SelectMgr_FilterType_OR);
  occ::handle<NCollection_Shared<NCollection_Map<const Standard_Transient*>>> blocked=new NCollection_Shared<NCollection_Map<const Standard_Transient*>>();
  if(disabled)blocked->Add(object.get());combined->SetDisabledObjects(blocked);
  for(int i=0;i<count;++i){occ::handle<V3AisFilter> filter=new V3AisFilter(selection.get(),owner);filter->mode=r.integer();callbacks.push_back(filter);
    andFilter->Add(filter);orFilter->Add(filter);combined->Add(filter);}
  GA_TRY(ga_flag(kind==0?andFilter->IsOk(owner):kind==1?orFilter->IsOk(owner):combined->IsOk(owner)))
  for(auto& filter:callbacks)ga_int(filter->observed.size()/3);ga_end();
}
