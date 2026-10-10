// Concrete triangular volume at OCCT 3d097a0; its protected constructor is exposed only for the fixture.
class V3TriangularFixture : public SelectMgr_TriangularFrustum { public: V3TriangularFixture() = default; };
occ::handle<Graphic3d_Camera> v3_triangle_camera(bool perspective,bool zero,double scale) {
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera;
  camera->SetProjectionType(perspective?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  camera->SetZeroToOneDepth(zero);camera->SetEyeAndCenter(gp_Pnt(0,0,10),gp::Origin());
  camera->SetZRange(.1,100.);camera->SetScale(scale);return camera;
}
gp_GTrsf v3_triangle_transform(GaReader& r) {
  gp_GTrsf transform;
  for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)transform.SetValue(row,col,r.real());
  return transform;
}
void v3_triangle_pick_state(const SelectBasics_PickResult& pick) {
  ga_real(pick.Depth());ga_real(pick.DistToGeomCenter());ga_pnt(pick.PickedPoint());
  auto normal=pick.SurfaceNormal();ga_real(normal.x());ga_real(normal.y());ga_real(normal.z());
}
void v3_intersector_query(SelectMgr_BaseIntersector& volume,GaReader& r,const SelectMgr_ViewClipRange& clip) {
  int kind=r.integer(),mode=r.integer();bool inside=mode==1;
  SelectBasics_PickResult pick(123.,456.,gp_Pnt(7,8,9));pick.SetSurfaceNormal(gp_Vec(1,2,3));
  if(kind==0){gp_XYZ a=r.xyz(),b=r.xyz();NCollection_Vec3<double> lo(a.X(),a.Y(),a.Z()),hi(b.X(),b.Y(),b.Z());
    GA_TRY(ga_flag(volume.OverlapsBox(lo,hi,mode<0?nullptr:&inside)))GA_TRY(ga_flag(volume.OverlapsBox(lo,hi,clip,pick)))}
  if(kind==1){gp_Pnt point=r.pnt();GA_TRY(ga_flag(volume.OverlapsPoint(point)))GA_TRY(ga_flag(volume.OverlapsPoint(point,clip,pick)))}
  if(kind==2){gp_Pnt a=r.pnt(),b=r.pnt();GA_TRY(ga_flag(volume.OverlapsSegment(a,b,clip,pick)))}
  if(kind==3){int sensitivity=r.integer();auto points=v3_frustum_points(r);GA_TRY(ga_flag(volume.OverlapsPolygon(points,static_cast<Select3D_TypeOfSensitivity>(sensitivity),clip,pick)))}
  if(kind==4){int sensitivity=r.integer();gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GA_TRY(ga_flag(volume.OverlapsTriangle(a,b,c,static_cast<Select3D_TypeOfSensitivity>(sensitivity),clip,pick)))}
  if(kind==5){gp_Pnt center=r.pnt();double radius=r.real();GA_TRY(ga_flag(volume.OverlapsSphere(center,radius,mode<0?nullptr:&inside)))GA_TRY(ga_flag(volume.OverlapsSphere(center,radius,clip,pick)))}
  if(kind==6){double bottom=r.real(),top=r.real(),height=r.real();bool hollow=r.flag();auto transform=v3_frustum_transform(r);GA_TRY(ga_flag(volume.OverlapsCylinder(bottom,top,height,transform,hollow,mode<0?nullptr:&inside)))GA_TRY(ga_flag(volume.OverlapsCylinder(bottom,top,height,transform,hollow,clip,pick)))}
  if(kind==7){double radius=r.real();bool filled=r.flag();auto transform=v3_frustum_transform(r);GA_TRY(ga_flag(volume.OverlapsCircle(radius,transform,filled,mode<0?nullptr:&inside)))GA_TRY(ga_flag(volume.OverlapsCircle(radius,transform,filled,clip,pick)))}
  ga_flag(inside);v3_triangle_pick_state(pick);
}
void v3_triangle_query(SelectMgr_TriangularFrustum& volume,GaReader& r) {
  SelectMgr_ViewClipRange clip;clip.ChangeUnclipRange()=Bnd_Range(200.,201.);
  v3_intersector_query(volume,r,clip);
}
void v3_triangle_state(SelectMgr_TriangularFrustum& volume,GaReader& r,bool cleared) {
  bool rebuild=r.flag();int depth=r.integer();if(rebuild)volume.Build();
  ga_flag(volume.IsScalable());ga_int(volume.GetSelectionType());
  if(!cleared){int width=0,height=0;volume.WindowSize(width,height);ga_int(width);ga_int(height);}
  NCollection_DynamicArray<NCollection_Vec4<double>> planes;planes.Append(NCollection_Vec4<double>(11,22,33,44));volume.GetPlanes(planes);
  ga_int(planes.Size());for(int i=0;i<planes.Size();++i){auto plane=planes.Value(i);ga_real(plane.x());ga_real(plane.y());ga_real(plane.z());ga_real(plane.w());}
  std::ostringstream stream;volume.DumpJson(stream,depth);ga_text(depth==0?stream.str():v3_frustum_dump_pointers(stream.str()));
}
void v3_triangular_case(std::istream& input) {
  GaReader r{input};bool perspective=r.flag(),zero=r.flag(),viewport=r.flag();
  auto camera=v3_triangle_camera(perspective,zero,20.);auto otherCamera=v3_triangle_camera(!perspective,zero,7.);
  occ::handle<SelectMgr_FrustumBuilder> builder=new SelectMgr_FrustumBuilder;
  builder->SetCamera(otherCamera);builder->SetWindowSize(320,240);
  occ::handle<SelectMgr_TriangularFrustum> volume=new V3TriangularFixture;
  volume->SetCamera(camera);volume->SetWindowSize(800,600);if(viewport)volume->SetViewport(.1,.2,.9,.8);
  gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();volume->Init(a,b,c);volume->Build();volume->SetPixelTolerance(9);
  int variant=r.integer(),scale=r.integer();auto transform=v3_triangle_transform(r);
  if(variant==1)volume=occ::down_cast<SelectMgr_TriangularFrustum>(volume->ScaleAndTransform(scale,transform,builder));
  if(variant==2||variant==3){volume=occ::down_cast<SelectMgr_TriangularFrustum>(volume->CopyWithBuilder(builder));if(variant==3)volume->Build();}
  if(variant==4)volume->Clear();
  ga_flag(volume->Camera()==camera);ga_flag(volume->Camera()==otherCamera);
  ga_flag(volume->IsKind("SelectMgr_BaseFrustum"));ga_flag(volume->IsKind("SelectMgr_BaseIntersector"));ga_flag(volume->IsInstance("SelectMgr_TriangularFrustum"));
  int family=r.integer();GA_TRY(if(family==0){v3_triangle_state(*volume,r,variant==4);}else{v3_triangle_query(*volume,r);})ga_end();
}
