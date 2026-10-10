// Calls the linked private overlap body; owner-slot replacement is a client callback.
struct V3CheckOverlapTag {
  using Type=void(SelectMgr_ViewerSelector::*)(const occ::handle<Select3D_SensitiveEntity>&,const gp_GTrsf&,SelectMgr_SelectingVolumeManager&);
  friend Type v3_selector_member(V3CheckOverlapTag);
};
template struct V3SelectorAccess<V3CheckOverlapTag,&SelectMgr_ViewerSelector::checkOverlap>;
class V3OverlapPoint : public Select3D_SensitivePoint {
public:
  SelectBasics_PickResult pick;int decision=1,matchOwner=-1,locationOwner=-1;
  std::vector<occ::handle<SelectMgr_EntityOwner>>& owners;mutable std::vector<int> events;
  V3OverlapPoint(std::vector<occ::handle<SelectMgr_EntityOwner>>& values):Select3D_SensitivePoint(nullptr,gp_Pnt()),owners(values){}
  bool Matches(SelectBasics_SelectingVolumeManager&,SelectBasics_PickResult& result) override {
    events.push_back(1);if(matchOwner>=0)Set(owners[matchOwner]);result=pick;
    if(decision==2)throw Standard_Failure("fixture match failure");return decision!=0;
  }
  bool HasInitLocation() const override {events.push_back(2);if(locationOwner>=0)const_cast<V3OverlapPoint*>(this)->Set(owners[locationOwner]);return false;}
};
class V3OverlapSelector : public SelectMgr_ViewerSelector {
public:
  V3OverlapSelector(){myZLayerOrderMap.Bind(10,4);myZLayerOrderMap.Bind(20,7);myZLayerOrderMap.Bind(0,2);myIsSorted=true;}
  void state(const std::vector<occ::handle<SelectMgr_EntityOwner>>& owners,const std::vector<occ::handle<V3OverlapPoint>>& entities){
    ga_flag(myIsSorted);ga_int(mystored.Size());
    for(int i=1;i<=mystored.Size();++i){for(int j=0;j<int(owners.size());++j)if(mystored.FindKey(i)==owners[j])ga_int(j);
      auto c=mystored.FindFromIndex(i);for(int j=0;j<2;++j)if(c.Entity==entities[j])ga_int(j);v3_criterion_state(c);}
    for(auto entity:entities){for(int j=0;j<int(owners.size());++j)if(entity->OwnerId()==owners[j])ga_int(j);
      ga_int(entity->events.size());for(int event:entity->events)ga_int(event);entity->events.clear();}
  }
};
static occ::handle<SelectMgr_SelectableObject> v3_overlap_object(const occ::handle<AIS_InteractiveContext>& ctx,int kind,int persistent,int presentation,int mode,int index){
  if(kind==0)return nullptr;occ::handle<SelectMgr_SelectableObject> object;
  if(kind==1)object=new V3BoxedSelectable(0.,false);
  else {occ::handle<V3ContextObject> ais=new V3ContextObject();ais->SetContext(ctx);object=ais;}
  object->SetZLayer(10+index*10);object->Attributes()->SetDisplayMode(mode);
  if(persistent){int flags=persistent==1?Graphic3d_TMF_2d:(persistent==2?Graphic3d_TMF_OrthoPers|Graphic3d_TMF_2d:Graphic3d_TMF_ZoomPers);
    object->SetTransformPersistence(new Graphic3d_TransformPers(Graphic3d_TransModeFlags(flags)));}
  if(presentation){auto prs=ctx->MainPrsMgr()->Presentation(object,0,true);prs->SetDisplayPriority(index?Graphic3d_DisplayPriority_Topmost:Graphic3d_DisplayPriority_Above);}
  return object;
}
void v3_selector_overlap(std::istream& input){
  GaReader r{input};int kind=r.integer(),persistent=r.integer(),presentation=r.integer(),mode=r.integer(),volumeKind=r.integer(),steps=r.integer();
  occ::handle<AIS_InteractiveContext> ctx=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));
  auto first=v3_overlap_object(ctx,kind,persistent,presentation,mode,0),second=v3_overlap_object(ctx,kind,persistent,presentation,mode,1);
  std::vector<occ::handle<SelectMgr_EntityOwner>> owners={nullptr,new SelectMgr_EntityOwner(first,3),new SelectMgr_EntityOwner(second,9),new SelectMgr_EntityOwner(7)};
  std::vector<occ::handle<V3OverlapPoint>> entities={new V3OverlapPoint(owners),new V3OverlapPoint(owners)};
  V3OverlapSelector selector;auto volume=v3_geometry_volume(volumeKind,1);selector.state(owners,entities);
  for(int i=0;i<steps;++i){int id=r.integer(),owner=r.integer(),matchOwner=r.integer(),locationOwner=r.integer(),decision=r.integer(),pickKind=r.integer(),transform=r.integer();double depth=r.real(),distance=r.real();
    auto entity=entities[id];entity->Set(owners[owner]);entity->matchOwner=matchOwner;entity->locationOwner=locationOwner;entity->decision=decision;entity->pick=SelectBasics_PickResult();
    if(pickKind)entity->pick.SetDepth(depth);entity->pick.SetDistToGeomCenter(distance);if(pickKind==2)entity->pick.SetPickedPoint(gp_Pnt(1.,2.,3.));
    GA_TRY((selector.*v3_selector_member(V3CheckOverlapTag{}))(entity,v3_geometry_transform(transform),volume))selector.state(owners,entities);
  }ga_end();
}
