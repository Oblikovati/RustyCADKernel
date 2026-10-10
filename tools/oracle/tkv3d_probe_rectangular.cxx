// OCCT 3d097a0 rectangular fixture; accessors expose protected helpers without new RTTI.
class V3RectangularFixture : public SelectMgr_RectangularFrustum {
public:
  using SelectMgr_RectangularFrustum::segmentSegmentDistance;
  using SelectMgr_RectangularFrustum::segmentPlaneIntersection;
};
void v3_rectangle_state(SelectMgr_RectangularFrustum& volume,GaReader& r) {
  bool rebuild=r.flag();int depth=r.integer();if(rebuild)volume.Build();
  ga_flag(volume.IsScalable());ga_int(volume.GetSelectionType());
  ga_pnt(volume.GetNearPnt());ga_pnt(volume.GetFarPnt());ga_xyz(volume.GetViewRayDirection().XYZ());ga_pnt2d(volume.GetMousePosition());
  GA_TRY(ga_real(volume.DistToGeometryCenter(gp_Pnt(1,2,3))))GA_TRY(ga_pnt(volume.DetectedPoint(7.)))
  for(int i=0;i<8;++i)ga_pnt(volume.GetVertices()[i]);
  if(!volume.Camera().IsNull()) {
    NCollection_DynamicArray<NCollection_Vec4<double>> planes;planes.Append(NCollection_Vec4<double>(11,22,33,44));volume.GetPlanes(planes);
    ga_int(planes.Size());for(int i=0;i<planes.Size();++i){auto p=planes.Value(i);ga_real(p.x());ga_real(p.y());ga_real(p.z());ga_real(p.w());}
  }
  std::ostringstream stream;volume.DumpJson(stream,depth);ga_text(depth==0?stream.str():v3_frustum_dump_pointers(stream.str()));
}
void v3_rectangle_protected(V3RectangularFixture& volume,GaReader& r) {
  int kind=r.integer();SelectBasics_PickResult pick(123.,456.,gp_Pnt(7,8,9));pick.SetSurfaceNormal(gp_Vec(1,2,3));
  if(kind==0){gp_Pnt a=r.pnt(),b=r.pnt();volume.segmentSegmentDistance(a,b,pick);}
  if(kind==1){gp_Vec normal(r.xyz());gp_Pnt p=r.pnt();ga_flag(volume.segmentPlaneIntersection(normal,p,pick));}
  if(kind==2){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt(),d=r.pnt();ga_flag(volume.isSegmentsIntersect(a,b,c,d));}
  if(kind==3){double radius=r.real();gp_Pnt center=r.pnt();auto transform=v3_frustum_transform(r);auto points=v3_frustum_points(r);GA_TRY(ga_flag(volume.isIntersectCircle(radius,center,transform,points)))}
  v3_triangle_pick_state(pick);
}
void v3_rectangle_bounds(GaReader& r) {
  SelectMgr_RectangularFrustum::SelectionRectangle bounds;
  ga_pnt2d(bounds.MinPnt());ga_pnt2d(bounds.MaxPnt());gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();
  bounds.SetMousePos(a);ga_pnt2d(bounds.MousePos());ga_pnt2d(bounds.MaxPnt());
  bounds.SetMinPnt(b);bounds.SetMaxPnt(c);ga_pnt2d(bounds.MousePos());ga_pnt2d(bounds.MinPnt());ga_pnt2d(bounds.MaxPnt());
}
void v3_rectangle_case(std::istream& input) {
  GaReader r{input};bool perspective=r.flag(),zero=r.flag(),viewport=r.flag();int selection=r.integer();gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();int tolerance=r.integer();bool build=r.flag();
  auto camera=v3_triangle_camera(perspective,zero,20.);auto otherCamera=v3_triangle_camera(!perspective,zero,7.);
  occ::handle<V3RectangularFixture> original=new V3RectangularFixture;original->SetCamera(camera);original->SetWindowSize(800,600);
  if(viewport)original->SetViewport(.1,.2,.9,.8);original->SetPixelTolerance(tolerance);
  if(selection==1)original->Init(a);if(selection==2)original->Init(a,b);
  GA_TRY(
    if(build)original->Build();
    int transition=r.integer(),scale=r.integer(),matrixMode=r.integer(),builderKind=r.integer();auto matrix=v3_triangle_transform(r);gp_GTrsf transform=matrixMode==0?gp_GTrsf():matrix;
    occ::handle<SelectMgr_FrustumBuilder> builder;
    if(builderKind){builder=new SelectMgr_FrustumBuilder;builder->SetCamera(builderKind==1?otherCamera:camera);builder->SetWindowSize(320,240);}
    occ::handle<SelectMgr_BaseIntersector> base=original;
    if(transition==1)base=base->ScaleAndTransform(scale,transform,builder);
    if(transition==2||transition==3){base=base->CopyWithBuilder(builder);if(transition==3)base->Build();}
    SelectMgr_RectangularFrustum& volume=*static_cast<SelectMgr_RectangularFrustum*>(base.get());
    ga_flag(volume.Camera()==camera);ga_flag(volume.Camera()==otherCamera);
    ga_text(volume.DynamicType()->Name());ga_text(SelectMgr_RectangularFrustum::get_type_descriptor()->Name());
    ga_flag(volume.IsKind("SelectMgr_RectangularFrustum"));ga_flag(volume.IsInstance("SelectMgr_BaseFrustum"));
    int family=r.integer();
    if(family==0)v3_rectangle_state(volume,r);
    if(family==1){auto clip=v3_axis_clip(r);v3_intersector_query(volume,r,clip);}
    if(family==2)v3_rectangle_protected(*original,r);
    if(family==3)v3_rectangle_bounds(r);
  )ga_end();
}
