// SelectMgr helpers at OCCT 3d097a0. Twin: v3_probe/selection_helpers.rs.
#include <SelectMgr_FrustumBuilder.hxx>
#include <SelectMgr_ViewClipRange.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
#include <Graphic3d_ClipPlane.hxx>
void v3_frustum_builder_case(std::istream& input) {
  GaReader r{input}; SelectMgr_FrustumBuilder builder;
  int width=0,height=0; builder.WindowSize(width,height); ga_int(width); ga_int(height);
  ga_flag(builder.Camera().IsNull()); ga_pnt(builder.ProjectPntOnViewPlane(1,2,0));
  bool present=r.flag(), perspective=r.flag(), zero=r.flag();
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera;
  camera->SetProjectionType(perspective?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  camera->SetZeroToOneDepth(zero); camera->SetZRange(0.1,100.);
  if(present) builder.SetCamera(camera);
  width=r.integer();height=r.integer();builder.SetWindowSize(width,height);
  bool viewport=r.flag();double vx=r.real(),vy=r.real(),vw=r.real(),vh=r.real();
  if(viewport)builder.SetViewport(vx,vy,vw,vh);
  double x=r.real(),y=r.real(),z=r.real();gp_XYZ eq=r.xyz(),point=r.xyz();
  ga_flag(builder.Camera()==camera);builder.WindowSize(width,height);ga_int(width);ga_int(height);
  ga_real(builder.SignedPlanePntDist(NCollection_Vec3<double>(eq.X(),eq.Y(),eq.Z()),NCollection_Vec3<double>(point.X(),point.Y(),point.Z())));
  ga_real(builder.SignedPlanePntDist(NCollection_Vec3<double>(0.,0.,0.),NCollection_Vec3<double>(point.X(),point.Y(),point.Z())));
  GA_TRY(ga_pnt(builder.ProjectPntOnViewPlane(x,y,z)))
  builder.InvalidateViewport();GA_TRY(ga_pnt(builder.ProjectPntOnViewPlane(x,y,z)))
  ga_end();
}
Graphic3d_SequenceOfHClipPlane v3_clip_planes(GaReader& r) {
  Graphic3d_SequenceOfHClipPlane sequence;occ::handle<Graphic3d_ClipPlane> previous;
  int count=r.integer();
  for(int i=0;i<count;++i) {
    double a=r.real(),b=r.real(),c=r.real(),d=r.real();bool on=r.flag(),linked=r.flag();
    occ::handle<Graphic3d_ClipPlane> plane=new Graphic3d_ClipPlane(Graphic3d_ClipPlane::Equation(a,b,c,d));
    plane->SetOn(on);if(linked)previous->SetChainNextPlane(plane);else sequence.Append(plane);
    previous=plane;
  }
  return sequence;
}
void v3_clip_queries(SelectMgr_ViewClipRange& clip,GaReader& r) {
  for(int n=r.integer();n>0;--n) {
    double first=r.real(),last=r.real(),depth=123.;Bnd_Range query;
    if(first<=last)query=Bnd_Range(first,last);
    ga_flag(clip.IsClipped(first));ga_flag(clip.IsClipped(last));
    ga_flag(clip.GetNearestDepth(query,depth));ga_real(depth);
  }
  ga_captured([&]{clip.DumpJson(std::cout,-1);});clip.SetVoid();ga_flag(clip.IsClipped(0.));
}
void v3_clip_range_case(std::istream& input) {
  GaReader r{input};SelectMgr_ViewClipRange clip;
  bool empty=r.flag();double first=r.real(),last=r.real();
  if(empty)clip.ChangeUnclipRange().SetVoid();else clip.ChangeUnclipRange()=Bnd_Range(first,last);
  for(int n=r.integer();n>0;--n){double a=r.real(),b=r.real();clip.AddClipSubRange(Bnd_Range(a,b));}
  auto planes=v3_clip_planes(r);gp_Pnt origin=r.pnt();gp_Dir direction=r.dir();
  GA_TRY(clip.AddClippingPlanes(planes,gp_Ax1(origin,direction));v3_clip_queries(clip,r))
  ga_end();
}
