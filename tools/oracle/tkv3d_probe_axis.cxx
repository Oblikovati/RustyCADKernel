// Native AxisIntersector fixture at OCCT 3d097a0, including inherited RTTI and protected helpers.
#include <SelectMgr_AxisIntersector.hxx>
class V3AxisFixture : public SelectMgr_AxisIntersector {
public:
  using SelectMgr_AxisIntersector::hasIntersection;
  using SelectMgr_AxisIntersector::raySegmentDistance;
  using SelectMgr_AxisIntersector::rayPlaneIntersection;
};
SelectMgr_ViewClipRange v3_axis_clip(GaReader& r) {
  SelectMgr_ViewClipRange clip;bool empty=r.flag();double first=r.real(),last=r.real();
  if(empty)clip.ChangeUnclipRange().SetVoid();else clip.ChangeUnclipRange()=Bnd_Range(first,last);
  for(int count=r.integer();count>0;--count){double a=r.real(),b=r.real();clip.AddClipSubRange(Bnd_Range(a,b));}
  return clip;
}
void v3_axis_state(SelectMgr_BaseIntersector& volume,GaReader& r) {
  int depth=r.integer();double distance=r.real();gp_Pnt point=r.pnt();
  volume.Build();volume.SetCamera(new Graphic3d_Camera);volume.SetPixelTolerance(17);volume.SetWindowSize(10,20);volume.SetViewport(.1,.2,.3,.4);
  ga_flag(volume.Camera().IsNull());ga_flag(volume.IsScalable());ga_int(volume.GetSelectionType());
  int w=123,h=456;volume.WindowSize(w,h);ga_int(w);ga_int(h);
  ga_text(volume.DynamicType()->Name());ga_text(SelectMgr_AxisIntersector::get_type_descriptor()->Name());
  ga_flag(volume.IsKind("SelectMgr_AxisIntersector"));ga_flag(volume.IsInstance("SelectMgr_BaseIntersector"));
  GA_TRY(ga_pnt(volume.GetNearPnt()))GA_TRY(ga_pnt(volume.GetFarPnt()))GA_TRY(ga_xyz(volume.GetViewRayDirection().XYZ()))
  ga_pnt2d(volume.GetMousePosition());GA_TRY(ga_real(volume.DistToGeometryCenter(point)))GA_TRY(ga_pnt(volume.DetectedPoint(distance)))
  std::ostringstream stream;volume.DumpJson(stream,depth);ga_text(stream.str());
}
void v3_axis_protected(V3AxisFixture& volume,GaReader& r) {
  int kind=r.integer();double entry=123.,leave=456.;
  SelectBasics_PickResult pick(123.,456.,gp_Pnt(7,8,9));pick.SetSurfaceNormal(gp_Vec(1,2,3));
  if(kind==0){gp_XYZ a=r.xyz(),b=r.xyz();GA_TRY(ga_flag(volume.hasIntersection(NCollection_Vec3<double>(a.X(),a.Y(),a.Z()),NCollection_Vec3<double>(b.X(),b.Y(),b.Z()),entry,leave)))}
  if(kind==1){gp_Pnt point=r.pnt();GA_TRY(ga_flag(volume.hasIntersection(point,entry)))}
  if(kind==2){gp_Pnt a=r.pnt(),b=r.pnt();ga_flag(volume.raySegmentDistance(a,b,pick));}
  if(kind==3){gp_Vec normal(r.xyz());gp_Pnt point=r.pnt();ga_flag(volume.rayPlaneIntersection(normal,point,pick));}
  ga_real(entry);ga_real(leave);v3_triangle_pick_state(pick);
}
void v3_axis_case(std::istream& input) {
  GaReader r{input};bool initialized=r.flag();gp_Pnt point=r.pnt();gp_Dir direction=r.dir();
  occ::handle<V3AxisFixture> original=new V3AxisFixture;if(initialized)original->Init(gp_Ax1(point,direction));
  occ::handle<SelectMgr_BaseIntersector> volume=original;int transition=r.integer(),matrixMode=r.integer(),scale=r.integer();
  auto matrix=v3_triangle_transform(r);gp_GTrsf transform=matrixMode==0?gp_GTrsf():matrix;
  GA_TRY(
    if(transition==1)volume=volume->ScaleAndTransform(scale,transform,nullptr);
    if(transition==2)volume=volume->CopyWithBuilder(nullptr);
    int family=r.integer();
    if(family==0)v3_axis_state(*volume,r);
    if(family==1){auto clip=v3_axis_clip(r);v3_intersector_query(*volume,r,clip);}
    if(family==2)v3_axis_protected(*original,r);
  )ga_end();
}
