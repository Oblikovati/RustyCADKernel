// Actual two-level BVH traversal over real points, with observable client callbacks.
struct V3TraverseObjectTag {
  using Type=void(SelectMgr_ViewerSelector::*)(const occ::handle<SelectMgr_SelectableObject>&,const SelectMgr_SelectingVolumeManager&,const occ::handle<Graphic3d_Camera>&,const NCollection_Mat4<double>&,const NCollection_Mat4<double>&,const NCollection_Vec2<int>&);
  friend Type v3_selector_member(V3TraverseObjectTag);
};
template struct V3SelectorAccess<V3TraverseObjectTag,&SelectMgr_ViewerSelector::traverseObject>;
struct V3TraversalConfig {int kind,camera,persistence,entityPersistence,transform,clipping,overlap,flags,failing,count;};
static occ::handle<Graphic3d_SequenceOfHClipPlane> v3_traversal_planes(double coefficient,double offset){
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane();planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(coefficient,0.,0.,offset)));return planes;
}
static occ::handle<Graphic3d_TransformPers> v3_traversal_persistence(int code){
  if(code==0)return nullptr;int modes[]={0,2,64,258,320,128};return new Graphic3d_TransformPers(Graphic3d_TransModeFlags(modes[code]));
}
static SelectMgr_SelectingVolumeManager v3_traversal_volume(V3TraversalConfig config){
  SelectMgr_SelectingVolumeManager volume;volume.AllowOverlapDetection(config.overlap!=0);
  switch(config.kind){
    case 0:return volume;
    case 1:volume.InitPointSelectingVolume(gp_Pnt2d(40.,30.));break;
    case 2:volume.InitBoxSelectingVolume(gp_Pnt2d(20.,15.),gp_Pnt2d(60.,45.));break;
    case 3:volume.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(0.,0.,20.),gp_Dir(0.,0.,-1.)));break;
    case 4:{NCollection_Array1<gp_Pnt2d> points(1,4);points(1)=gp_Pnt2d(20.,15.);points(2)=gp_Pnt2d(60.,15.);points(3)=gp_Pnt2d(60.,45.);points(4)=gp_Pnt2d(20.,45.);volume.InitPolylineSelectingVolume(points);break;}
  }
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();camera->SetScale(20.);camera->SetAspect(4./3.);camera->SetEyeAndCenter(gp_Pnt(0.,0.,20.),gp_Pnt());
  if(config.camera==2)camera->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  volume.SetCamera(camera);volume.SetWindowSize(80,60);volume.SetPixelTolerance(3);volume.BuildSelectingVolume();
  if(config.clipping==1||config.clipping==3)volume.SetViewClipping(v3_traversal_planes(1.,0.),nullptr,nullptr);
  if(config.camera==0)volume.SetCamera(nullptr);return volume;
}
class V3TraversalObject : public SelectMgr_SelectableObject {
public:
  void BoundingBox(Bnd_Box& result) override {Bnd_Box local;local.Update(-2.,-1.,0.,8.,1.,0.);result=local.Transformed(Transformation());}
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>&,int) override {}
  void ComputeSelection(const occ::handle<SelectMgr_Selection>&,int) override {}
};
class V3TraversalPoint : public Select3D_SensitivePoint {
public:
  int tag,fail=0;std::vector<int>& events;SelectMgr_ViewerSelector* selector;
  V3TraversalPoint(const occ::handle<SelectMgr_EntityOwner>& owner,int id,std::vector<int>& trace,SelectMgr_ViewerSelector* source)
    :Select3D_SensitivePoint(owner,gp_Pnt(id%3==0?-2.:(id%3==1?0.:8.),id%3==2?-1.:0.,0.)),tag(id),events(trace),selector(source){}
  bool Matches(SelectBasics_SelectingVolumeManager& volume,SelectBasics_PickResult& pick) override {
    events.push_back(tag);(void)selector->NbPicked();if(fail)throw Standard_Failure("fixture match failure");return Select3D_SensitivePoint::Matches(volume,pick);
  }
};
class V3TraversalSelector : public SelectMgr_ViewerSelector {
public:
  using SelectMgr_ViewerSelector::TraverseSensitives;mutable int sorts=0;
  void SortResult() const override {++sorts;SelectMgr_ViewerSelector::SortResult();}
  void volume(const SelectMgr_SelectingVolumeManager& value){mySelectingVolumeMgr=value;myTolerances.Add(3);}
  const SelectMgr_SelectingVolumeManager& volume(){return mySelectingVolumeMgr;}
  int layerCount() const {return myZLayerOrderMap.Extent();}
  void state(const std::vector<occ::handle<SelectMgr_EntityOwner>>& owners){
    ga_int(sorts);ga_flag(myIsSorted);ga_pnt(myCameraEye);ga_dir(myCameraDir);ga_real(myCameraScale);ga_int(mystored.Size());
    for(int i=1;i<=mystored.Size();++i){for(int j=0;j<int(owners.size());++j)if(mystored.FindKey(i)==owners[j])ga_int(j);
      auto criterion=mystored.FindFromIndex(i);ga_int(static_cast<V3TraversalPoint*>(criterion.Entity.get())->tag);v3_criterion_state(criterion);}
    ga_int(myIndexes.Size());for(int index:myIndexes)ga_int(index);
  }
};
static void v3_configure_traversal_object(const occ::handle<V3TraversalObject>& object,V3TraversalConfig config,int index){
  object->SetTransformPersistence(v3_traversal_persistence(config.persistence==6?index%6:config.persistence));
  gp_Trsf transform;if(config.transform)transform.SetRotation(gp_Ax1(gp_Pnt(),gp::DZ()),1.8);transform.SetTranslationPart(gp_Vec((index%3-1)*10.,0.,0.));object->SetLocalTransformation(transform);
  if((config.flags&4)&&index%2==0)object->ViewAffinity()->SetVisible(2,false);
  if(config.clipping==2||config.clipping==3){auto planes=v3_traversal_planes(-1.,4.);planes->SetOverrideGlobal(config.clipping==3);object->SetClipPlanes(planes);}
}
class V3TraversalScene {
public:
  std::vector<int> events;occ::handle<V3TraversalSelector> selector=new V3TraversalSelector();
  std::vector<occ::handle<V3TraversalObject>> objects;std::vector<occ::handle<V3TraversalPoint>> entities;std::vector<occ::handle<SelectMgr_EntityOwner>> owners;
  V3TraversalScene(V3TraversalConfig config){selector->volume(v3_traversal_volume(config));
    for(int index=0;index<config.count;++index)addObject(config,index);if(config.flags&8)selector->SetToPrebuildBVH(true,2);
  }
  void addObject(V3TraversalConfig config,int index){
    occ::handle<V3TraversalObject> object=new V3TraversalObject();v3_configure_traversal_object(object,config,index);
    occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(object,index%4);owners.push_back(owner);occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
    for(int part=0;part<3;++part){int tag=index*3+part;occ::handle<V3TraversalPoint> entity=new V3TraversalPoint(owner,tag,events,selector.get());
      entity->SetSensitivityFactor(part+1);entity->fail=config.failing==tag;
      if(part==1)entity->SetTransformPersistence(v3_traversal_persistence(config.entityPersistence));
      if((config.flags&1)&&part==2)entity->SetFlippingOptions(true,gp_Ax2(gp_Pnt(),gp_Dir(0.,0.,-1.)));
      selection->Add(entity);entities.push_back(entity);
    }
    object->AddSelection(selection,0);selector->AddSelectionToObject(object,selection);
    int part=0;for(auto entity:selection->Entities()){if(!(config.flags&2)||part!=2)entity->SetActiveForSelection();++part;}
    selection->SetSelectionState(SelectMgr_SOS_Activated);objects.push_back(object);
  }
  void traverse(int entry,int viewId){
    if(entry!=0){selector->TraverseSensitives(viewId);return;}
    auto volume=selector->volume();auto camera=volume.Camera();NCollection_Mat4<double> projection,world;
    if(!camera.IsNull()){projection=camera->ProjectionMatrix();world=camera->OrientationMatrix();}
    NCollection_Vec2<int> size;volume.WindowSize(size.x(),size.y());
    for(auto object:objects)(selector.get()->*v3_selector_member(V3TraverseObjectTag{}))(object,volume,camera,projection,world,size);
    selector->SortResult();
  }
  void state(){selector->state(owners);ga_int(events.size());for(int event:events)ga_int(event);events.clear();}
};
void v3_selector_traversal(std::istream& input){
  GaReader r{input};V3TraversalConfig config{r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer(),r.integer()};int entry=r.integer(),viewId=r.integer();
  V3TraversalScene scene(config);GA_TRY(scene.traverse(entry,viewId))scene.state();
  if(entry==2){for(auto entity:scene.entities)entity->fail=0;GA_TRY(scene.traverse(entry,viewId))scene.state();}ga_end();
}
