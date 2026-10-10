// Stateful selecting manager oracle; inputs match selecting_volume.rs.
#include <SelectMgr_SelectingVolumeManager.hxx>
void v3_manager_init(SelectMgr_SelectingVolumeManager& m,int kind) {
  if(kind==0)m.InitSelectingVolume(nullptr);
  if(kind==1)m.InitPointSelectingVolume(gp_Pnt2d(400,300));
  if(kind==2)m.InitBoxSelectingVolume(gp_Pnt2d(200,100),gp_Pnt2d(600,500));
  if(kind==3){NCollection_Array1<gp_Pnt2d> p(1,4);p(1)=gp_Pnt2d(200,100);p(2)=gp_Pnt2d(600,100);p(3)=gp_Pnt2d(600,500);p(4)=gp_Pnt2d(200,500);m.InitPolylineSelectingVolume(p);}
  if(kind==4)m.InitAxisSelectingVolume(gp_Ax1(gp_Pnt(0,0,10),gp_Dir(0,0,-1)));
  if(kind==5)m.InitSelectingVolume(new SelectMgr_AxisIntersector);
}
occ::handle<Graphic3d_SequenceOfHClipPlane> v3_manager_plane(double z) {
  occ::handle<Graphic3d_SequenceOfHClipPlane> p=new Graphic3d_SequenceOfHClipPlane;
  p->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(0,0,1,-z)));return p;
}
void v3_manager_clips(SelectMgr_SelectingVolumeManager& m,int mode,int worldKind) {
  SelectMgr_ViewClipRange range;range.AddClipSubRange(Bnd_Range(2,4));m.SetViewClipRanges(range);if(!mode)return;
  SelectMgr_SelectingVolumeManager world;v3_manager_init(world,worldKind);
  occ::handle<Graphic3d_SequenceOfHClipPlane> view,object;if(mode>=2)view=v3_manager_plane(5);if(mode>=3)object=v3_manager_plane(6);
  GA_TRY(m.SetViewClipping(view,object,worldKind<0?nullptr:&world))
  if(mode==4){v3_manager_init(m,2);if(m.Camera().IsNull()){m.SetCamera(v3_triangle_camera(false,false,20));m.SetWindowSize(800,600);}m.SetViewClipping(nullptr,nullptr,nullptr);}
}
SelectMgr_SelectingVolumeManager v3_manager_copy(const SelectMgr_SelectingVolumeManager& m,int mode) {
  if(!mode)return m;occ::handle<SelectMgr_FrustumBuilder> b=new SelectMgr_FrustumBuilder;b->SetCamera(v3_triangle_camera(false,false,7));b->SetWindowSize(320,240);
  if(mode==1)return m.CopyWithBuilder(b);if(mode==2)return m.CopyWithBuilder(nullptr);return m.ScaleAndTransform(20,gp_GTrsf(),b);
}
void v3_manager_dump(const SelectMgr_SelectingVolumeManager& m,int depth) {
  std::ostringstream stream;m.DumpJson(stream,depth);std::string text=stream.str();
  for(const std::string field:{"ActiveSelectingVolume","ViewClipPlanes","ObjectClipPlanes"}){
    std::string prefix="\""+field+"\": \"";auto start=text.find(prefix)+prefix.size();auto end=text.find('"',start);
    text.replace(start,end-start,end==start?"null":"present");}ga_text(text);
}
void v3_manager_state(const SelectMgr_SelectingVolumeManager& m,int depth) {
  ga_int(m.GetActiveSelectionType());ga_flag(m.IsOverlapAllowed());ga_flag(m.IsScalableActiveVolume());
  int w=123,h=456;m.WindowSize(w,h);ga_int(w);ga_int(h);ga_pnt2d(m.GetMousePosition());
  GA_TRY(ga_pnt(m.GetNearPickedPnt()))GA_TRY(ga_pnt(m.GetFarPickedPnt()))GA_TRY(ga_xyz(m.GetViewRayDirection().XYZ()))
  GA_TRY(ga_pnt(m.DetectedPoint(3)))GA_TRY(ga_real(m.DistToGeometryCenter(gp_Pnt(1,2,3))))
  NCollection_DynamicArray<NCollection_Vec4<double>> planes;planes.Append(NCollection_Vec4<double>(11,22,33,44));m.GetPlanes(planes);ga_int(planes.Size());
  for(int i=0;i<planes.Size();++i){auto p=planes.Value(i);ga_real(p.x());ga_real(p.y());ga_real(p.z());ga_real(p.w());}
  for(double d:{-10.,0.,2.,3.,4.,5.,6.,10.,100.})ga_flag(m.ViewClipRanges().IsClipped(d));v3_manager_dump(m,depth);
}
void v3_manager_linear(const SelectMgr_SelectingVolumeManager& m,gp_Pnt point,double radius,SelectBasics_PickResult& pick) {
  NCollection_Vec3<double> lo(point.X()-radius,point.Y()-radius,point.Z()-radius),hi(point.X()+radius,point.Y()+radius,point.Z()+radius);bool inside=true;
  GA_TRY(ga_flag(m.OverlapsBox(lo,hi,&inside)))ga_flag(inside);GA_TRY(ga_flag(m.OverlapsBox(lo,hi,pick)))v3_triangle_pick_state(pick);
  GA_TRY(ga_flag(m.OverlapsPoint(point)))GA_TRY(ga_flag(m.OverlapsPoint(point,pick)))v3_triangle_pick_state(pick);
  gp_Pnt end(point.X()+radius,point.Y(),point.Z()),third(point.X(),point.Y()+radius,point.Z());
  GA_TRY(ga_flag(m.OverlapsSegment(point,end,pick)))v3_triangle_pick_state(pick);
  GA_TRY(ga_flag(m.OverlapsTriangle(point,end,third,0,pick)))v3_triangle_pick_state(pick);
  NCollection_Array1<gp_Pnt> points(-2,0);points(-2)=point;points(-1)=end;points(0)=third;
  GA_TRY(ga_flag(m.OverlapsPolygon(points,1,pick)))v3_triangle_pick_state(pick);
}
void v3_manager_quadric(const SelectMgr_SelectingVolumeManager& m,gp_Pnt point,double radius,SelectBasics_PickResult& pick) {
  bool inside=true;gp_Trsf trsf;trsf.SetTranslation(gp_Vec(point.XYZ()));
  GA_TRY(ga_flag(m.OverlapsSphere(point,radius,&inside)))ga_flag(inside);GA_TRY(ga_flag(m.OverlapsSphere(point,radius,pick)))v3_triangle_pick_state(pick);
  GA_TRY(ga_flag(m.OverlapsCylinder(radius,0,3,trsf,false,&inside)))ga_flag(inside);GA_TRY(ga_flag(m.OverlapsCylinder(radius,0,3,trsf,false,pick)))v3_triangle_pick_state(pick);
  GA_TRY(ga_flag(m.OverlapsCircle(radius,trsf,true,&inside)))ga_flag(inside);GA_TRY(ga_flag(m.OverlapsCircle(radius,trsf,true,pick)))v3_triangle_pick_state(pick);
}
void v3_selecting_volume_case(std::istream& input) {
  GaReader r{input};int kind=r.integer();bool built=r.flag();int next=r.integer();bool allow=r.flag(),late=r.flag();int copy=r.integer(),clip=r.integer(),world=r.integer(),depth=r.integer();gp_Pnt point=r.pnt();double radius=r.real();
  GA_TRY(
    SelectMgr_SelectingVolumeManager m;m.AllowOverlapDetection(allow);v3_manager_init(m,kind);auto camera=v3_triangle_camera(false,false,20);
    GA_TRY(m.SetCamera(camera))GA_TRY(m.SetWindowSize(800,600))GA_TRY(m.SetPixelTolerance(20))GA_TRY(m.SetViewport(0,0,1,1))
    if(built){GA_TRY(m.BuildSelectingVolume())}auto old=m.ActiveVolume();v3_manager_init(m,next);ga_flag(old==m.ActiveVolume());ga_flag(m.Camera()==camera);
    if(!m.ActiveVolume().IsNull()&&m.Camera().IsNull()){m.SetCamera(camera);m.SetWindowSize(800,600);if(built){GA_TRY(m.BuildSelectingVolume())}}
    m.AllowOverlapDetection(late);v3_manager_clips(m,clip,world);auto result=v3_manager_copy(m,copy);v3_manager_state(result,depth);
    SelectBasics_PickResult pick(123,456,gp_Pnt(7,8,9));pick.SetSurfaceNormal(gp_Vec(1,2,3));v3_manager_linear(result,point,radius,pick);v3_manager_quadric(result,point,radius,pick);
  )ga_end();
}
