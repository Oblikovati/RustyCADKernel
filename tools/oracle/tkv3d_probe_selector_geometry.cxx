// Explicit instantiation exposes native member pointers without copying private bodies.
// C++ permits private names in explicit template-instantiation arguments.
template<class Tag,typename Tag::Type Member> struct V3SelectorAccess {
  friend typename Tag::Type v3_selector_member(Tag){return Member;}
};
struct V3UpdatePointTag {
  using Type=void(SelectMgr_ViewerSelector::*)(SelectMgr_SortCriterion&,const SelectBasics_PickResult&,const occ::handle<Select3D_SensitiveEntity>&,const gp_GTrsf&,const SelectMgr_SelectingVolumeManager&) const;
  friend Type v3_selector_member(V3UpdatePointTag);
};
struct V3ComputeFrustumTag {
  using Type=void(SelectMgr_ViewerSelector::*)(const occ::handle<Select3D_SensitiveEntity>&,const SelectMgr_SelectingVolumeManager&,const SelectMgr_SelectingVolumeManager&,const gp_GTrsf&,NCollection_DataMap<int,SelectMgr_SelectingVolumeManager>&,SelectMgr_SelectingVolumeManager&);
  friend Type v3_selector_member(V3ComputeFrustumTag);
};
template struct V3SelectorAccess<V3UpdatePointTag,&SelectMgr_ViewerSelector::updatePoint3d>;
template struct V3SelectorAccess<V3ComputeFrustumTag,&SelectMgr_ViewerSelector::computeFrustum>;
static gp_GTrsf v3_geometry_transform(int code){
  if(code==0)return gp_GTrsf();if(code==1){gp_Trsf transform;transform.SetTranslation(gp_Vec(2.,-3.,4.));return transform;}
  if(code==2)return gp_GTrsf(gp_Mat(2.,0.,0.,0.,3.,0.,0.,0.,4.),gp_XYZ(1.,2.,3.));
  if(code==3)return gp_GTrsf(gp_Mat(1.,0.5,0.,0.,2.,0.25,0.,0.,3.),gp_XYZ(-2.,1.,4.));
  return gp_GTrsf(gp_Mat(0.,0.,0.,0.,0.,0.,0.,0.,0.),gp_XYZ(1.,2.,3.));
}
class V3GeometryPoint : public Select3D_SensitivePoint {
public:
  bool initial;gp_GTrsf inverse;
  V3GeometryPoint(int code):Select3D_SensitivePoint(nullptr,gp_Pnt()),initial(code>=0),inverse(v3_geometry_transform(std::max(code,0))){}
  bool HasInitLocation() const override{return initial;}
  gp_GTrsf InvInitLocation() const override{return inverse;}
};
class V3GeometrySelector : public SelectMgr_ViewerSelector {
public:
  using SelectMgr_ViewerSelector::updateZLayers;
  void configure(int kind,double tolerance,double scale){myDepthTolType=SelectMgr_TypeOfDepthTolerance(kind);myDepthTolerance=tolerance;myCameraScale=scale;myCameraEye=gp_Pnt(1.,2.,3.);myCameraDir=gp_Dir(0.,0.,1.);}
  void volume(const SelectMgr_SelectingVolumeManager& manager,int adaptive,int custom){mySelectingVolumeMgr=manager;myTolerances.Add(adaptive);myTolerances.SetCustomTolerance(custom);}
  void layers(const occ::handle<V3d_View>& view){myZLayerOrderMap.Bind(99,88);updateZLayers(view);ga_flag(myZLayerOrderMap.IsBound(99));NCollection_Sequence<int> layers;view->Viewer()->GetAllZLayers(layers);ga_int(layers.Size());for(int layer:layers){ga_int(layer);ga_int(myZLayerOrderMap.Find(layer));}}
};
static SelectMgr_SelectingVolumeManager v3_geometry_volume(int kind,int projection){
  SelectMgr_SelectingVolumeManager volume;if(kind==0)return volume;
  if(kind==1)volume.InitPointSelectingVolume(gp_Pnt2d(4.,3.));
  if(kind==2)volume.InitBoxSelectingVolume(gp_Pnt2d(1.,1.),gp_Pnt2d(7.,5.));
  if(kind==3)volume.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(1.,2.,3.),gp_Dir(0.,0.,1.)));
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera();camera->SetProjectionType(projection==2?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  volume.SetCamera(camera);volume.SetWindowSize(8,6);volume.SetPixelTolerance(10);volume.BuildSelectingVolume();
  if(projection==0)volume.SetCamera(nullptr);return volume;
}
void v3_selector_point(std::istream& input){
  GaReader r{input};int kind=r.integer(),projection=r.integer(),pickKind=r.integer(),transform=r.integer(),initial=r.integer(),factor=r.integer(),tolKind=r.integer();
  float nx=r.real(),ny=r.real(),nz=r.real();double tolerance=r.real(),scale=r.real(),depth=r.real();
  V3GeometrySelector selector;selector.configure(tolKind,tolerance,scale);auto volume=v3_geometry_volume(kind,projection);
  occ::handle<V3GeometryPoint> located=new V3GeometryPoint(initial),argument=new V3GeometryPoint(-1);argument->SetSensitivityFactor(factor);
  SelectMgr_SortCriterion criterion;criterion.Entity=located;criterion.Point=gp_Pnt(8.,9.,10.);criterion.Normal=NCollection_Vec3<float>(3.,4.,5.);criterion.Tolerance=7.;criterion.Depth=depth;
  SelectBasics_PickResult pick;if(pickKind==1||pickKind==2)pick.SetDepth(100.);if(pickKind>=2)pick.SetPickedPoint(gp_Pnt(5.,8.,11.));pick.SetSurfaceNormal(NCollection_Vec3<float>(nx,ny,nz));
  GA_TRY((selector.*v3_selector_member(V3UpdatePointTag{}))(criterion,pick,argument,v3_geometry_transform(transform),volume))
  v3_criterion_state(criterion);ga_end();
}
static void v3_geometry_volume_state(const SelectMgr_SelectingVolumeManager& volume){
  ga_int(volume.GetActiveSelectionType());ga_flag(volume.Camera().IsNull());ga_flag(volume.ViewClipping().IsNull());ga_flag(volume.ObjectClipping().IsNull());
  for(double depth:{1.,2.,3.,4.,5.})ga_flag(volume.ViewClipRanges().IsClipped(depth));
  if(!volume.ActiveVolume().IsNull()){GA_TRY(ga_pnt(volume.DetectedPoint(2.)))}
}
void v3_selector_frustum(std::istream& input){
  GaReader r{input};int kind=r.integer(),projection=r.integer(),initial=r.integer(),transform=r.integer(),factor=r.integer(),adaptive=r.integer(),custom=r.integer();
  V3GeometrySelector selector;auto global=v3_geometry_volume(kind,projection),object=v3_geometry_volume(kind,projection);selector.volume(global,adaptive,custom);
  occ::handle<V3GeometryPoint> entity=new V3GeometryPoint(initial);entity->SetSensitivityFactor(factor);
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane();occ::handle<Graphic3d_SequenceOfHClipPlane> emptyPlanes;object.SetViewClipping(planes,emptyPlanes,nullptr);
  SelectMgr_ViewClipRange range;range.ChangeUnclipRange()=Bnd_Range(2.,4.);object.SetViewClipRanges(range);
  NCollection_DataMap<int,SelectMgr_SelectingVolumeManager> cache;SelectMgr_SelectingVolumeManager result;
  auto compute=[&](){(selector.*v3_selector_member(V3ComputeFrustumTag{}))(entity,global,object,v3_geometry_transform(transform),cache,result);};
  GA_TRY(compute())ga_int(cache.Size());ga_flag(result.ActiveVolume()==object.ActiveVolume());v3_geometry_volume_state(result);
  auto first=result.ActiveVolume();object.SetViewClipping(emptyPlanes,emptyPlanes,nullptr);GA_TRY(compute())
  ga_flag(result.ActiveVolume()==first);ga_int(cache.Size());v3_geometry_volume_state(result);
  for(NCollection_DataMap<int,SelectMgr_SelectingVolumeManager>::Iterator it(cache);it.More();it.Next()){ga_int(it.Key());v3_geometry_volume_state(it.Value());}ga_end();
}
void v3_selector_layers(std::istream& input){
  GaReader r{input};int seed=r.integer();occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());auto view=viewer->CreateView();V3GeometrySelector selector;
  NCollection_Sequence<int> layers;viewer->GetAllZLayers(layers);int index=0;
  for(int layer:layers){auto settings=viewer->ZLayerSettings(layer);settings.SetClearDepth((seed&(1<<index))!=0);settings.SetEnableDepthWrite((seed&(1<<(index+5)))!=0);viewer->SetZLayerSettings(layer,settings);++index;}
  selector.layers(view);ga_end();
}
class V3DebugSelector : public SelectMgr_ViewerSelector {
public:
  void append(const occ::handle<Graphic3d_Structure>& structure){myStructs.Append(structure);}
  int count(){return myStructs.Size();}
};
void v3_selector_clear(std::istream& input){
  GaReader r{input};int count=r.integer(),withView=r.integer();occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());V3DebugSelector selector;
  std::vector<occ::handle<Graphic3d_Structure>> structures;
  for(int i=0;i<count;++i){occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(viewer->StructureManager());structure->NewGroup();structure->Display();selector.append(structure);structures.push_back(structure);}
  auto view=withView?viewer->CreateView():occ::handle<V3d_View>();selector.ClearSensitive(view);ga_int(selector.count());
  for(auto& structure:structures){ga_flag(structure->IsDeleted());ga_flag(structure->IsDisplayed());}selector.ClearSensitive(view);ga_int(selector.count());ga_end();
}
