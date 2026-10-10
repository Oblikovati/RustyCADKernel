// Native polyline selecting-volume fixture; child order comes from the pinned Delaunay mesh.
#include <SelectMgr_TriangularFrustumSet.hxx>
NCollection_Array1<gp_Pnt2d> v3_set_points(GaReader& r) {
  int lower=r.integer(),count=r.integer();NCollection_Array1<gp_Pnt2d> points(lower,lower+count-1);
  for(int i=lower;i<=points.Upper();++i)points.SetValue(i,r.pnt2d());return points;
}
std::string v3_set_dump_pointers(std::string text) {
  for(const std::string field:{"Builder","Camera"}) {
    std::string prefix="\""+field+"\": \"";size_t offset=0;
    while(true){auto start=text.find(prefix,offset);if(start==std::string::npos)break;
      start+=prefix.size();auto end=text.find('"',start);std::string value=end==start?"null":"present";
      text.replace(start,end-start,value);offset=start+value.size();}
  }return text;
}
void v3_set_state(SelectMgr_TriangularFrustumSet& volume,GaReader& r) {
  bool rebuild=r.flag();int depth=r.integer();if(rebuild)volume.Build();
  ga_flag(volume.IsScalable());ga_int(volume.GetSelectionType());ga_pnt2d(volume.GetMousePosition());
  GA_TRY(ga_pnt(volume.DetectedPoint(3.)))
  NCollection_DynamicArray<NCollection_Vec4<double>> planes;planes.Append(NCollection_Vec4<double>(11,22,33,44));volume.GetPlanes(planes);
  ga_int(planes.Size());for(int i=0;i<planes.Size();++i){auto p=planes.Value(i);ga_real(p.x());ga_real(p.y());ga_real(p.z());ga_real(p.w());}
  std::ostringstream stream;volume.DumpJson(stream,depth);ga_text(v3_set_dump_pointers(stream.str()));
}
void v3_frustum_set_case(std::istream& input) {
  GaReader r{input};bool perspective=r.flag(),zero=r.flag(),viewport=r.flag(),allow=r.flag(),initialized=r.flag(),build=r.flag();
  GA_TRY(
    auto points=v3_set_points(r);auto camera=v3_triangle_camera(perspective,zero,20.);auto otherCamera=v3_triangle_camera(!perspective,zero,7.);
    occ::handle<SelectMgr_TriangularFrustumSet> original=new SelectMgr_TriangularFrustumSet;
    original->SetCamera(camera);original->SetWindowSize(800,600);if(viewport)original->SetViewport(.1,.2,.9,.8);
    original->SetPixelTolerance(17);original->SetAllowOverlapDetection(allow);if(initialized)original->Init(points);if(build)original->Build();
    int transition=r.integer(),scale=r.integer(),builderKind=r.integer();auto transform=v3_triangle_transform(r);
    occ::handle<SelectMgr_FrustumBuilder> builder;if(builderKind){builder=new SelectMgr_FrustumBuilder;builder->SetCamera(builderKind==1?otherCamera:camera);builder->SetWindowSize(320,240);}
    occ::handle<SelectMgr_BaseIntersector> base=original;
    if(transition==1)base=base->ScaleAndTransform(scale,transform,builder);
    if(transition>=2)base=base->CopyWithBuilder(builder);
    if(transition>=4){for(int i=points.Lower();i<=points.Upper();++i)points.ChangeValue(i).SetX(points.Value(i).X()+80.);original->Init(points);}
    if(transition==3||transition==4)base->Build();
    auto& volume=*static_cast<SelectMgr_TriangularFrustumSet*>(base.get());
    ga_flag(volume.Camera()==camera);ga_flag(volume.Camera()==otherCamera);
    ga_text(volume.DynamicType()->Name());ga_text(SelectMgr_TriangularFrustumSet::get_type_descriptor()->Name());
    ga_flag(volume.IsKind("SelectMgr_TriangularFrustumSet"));ga_flag(volume.IsInstance("SelectMgr_BaseFrustum"));
    int family=r.integer();if(family==0)v3_set_state(volume,r);else {auto clip=v3_axis_clip(r);v3_intersector_query(volume,r,clip);}
  )ga_end();
}
