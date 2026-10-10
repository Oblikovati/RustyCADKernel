// Shared frustum state and boundary sphere test at OCCT 3d097a0.
#include <SelectMgr_RectangularFrustum.hxx>
class V3FrustumProbe : public SelectMgr_RectangularFrustum {
public:
  bool rejectSphereRays=false;
  const occ::handle<SelectMgr_FrustumBuilder>& Builder() const { return myBuilder; }
  int Tolerance() const { return myPixelTolerance; }
  bool RaySphereIntersection(const gp_Pnt& center,double radius,const gp_Pnt& origin,
      const gp_Dir& direction,double& enter,double& leave) const override {
    return !rejectSphereRays && SelectMgr_BaseIntersector::RaySphereIntersection(center,radius,origin,direction,enter,leave);
  }
};
void v3_base_frustum_camera(V3FrustumProbe& volume,GaReader& r) {
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera;
  camera->SetProjectionType(r.flag()?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  camera->SetZeroToOneDepth(r.flag());camera->SetZRange(.1,100.);
  volume.SelectMgr_BaseFrustum::SetCamera(camera);
  ga_flag(volume.Camera()==camera);ga_flag(volume.Builder()->Camera()==camera);
  volume.SelectMgr_BaseFrustum::SetViewport(.1,.2,.9,.8);
  ga_pnt(volume.Builder()->ProjectPntOnViewPlane(123,231,.2));
  volume.SelectMgr_BaseFrustum::SetCamera(camera);
  ga_pnt(volume.Builder()->ProjectPntOnViewPlane(123,231,.2));
  occ::handle<SelectMgr_FrustumBuilder> replacement=new SelectMgr_FrustumBuilder;
  replacement->SetWindowSize(321,123);volume.SetBuilder(replacement);
  ga_flag(volume.Camera().IsNull());ga_flag(volume.Builder()==replacement);
  volume.SelectMgr_BaseFrustum::SetCamera(camera);volume.SetBuilder(nullptr);
  ga_flag(volume.Camera()==camera);ga_flag(volume.Builder().IsNull());
  volume.SelectMgr_BaseFrustum::SetCamera(nullptr);ga_flag(volume.Camera().IsNull());
}
void v3_base_frustum_case(std::istream& input) {
  GaReader r{input};V3FrustumProbe volume,other;
  int w=0,h=0;volume.SelectMgr_BaseFrustum::WindowSize(w,h);ga_int(w);ga_int(h);
  ga_int(volume.Tolerance());ga_flag(volume.Builder()!=other.Builder());
  volume.SelectMgr_BaseFrustum::SetPixelTolerance(r.integer());
  w=r.integer();h=r.integer();volume.SelectMgr_BaseFrustum::SetWindowSize(w,h);
  volume.SelectMgr_BaseFrustum::WindowSize(w,h);ga_int(w);ga_int(h);ga_int(volume.Tolerance());
  v3_base_frustum_camera(volume,r);
  // Rust DumpStream is readable, matching ostringstream rather than an opaque cout.
  int depth=r.integer();std::ostringstream dump;volume.SelectMgr_BaseFrustum::DumpJson(dump,depth);ga_text(dump.str());ga_end();
}
void v3_boundary_sphere_case(std::istream& input) {
  GaReader r{input};V3FrustumProbe volume;volume.rejectSphereRays=r.flag();
  gp_Pnt center=r.pnt();double radius=r.real();gp_Dir normal=r.dir();bool inside=r.flag();
  int lower=r.integer(),count=r.integer();NCollection_Array1<gp_Pnt> points;
  if(count>0)points.Resize(lower,lower+count-1,false);
  for(int i=0;i<count;++i)points.SetValue(lower+i,r.pnt());
  GA_TRY(ga_flag(volume.IsBoundaryIntersectSphere(center,radius,normal,points,inside)))
  ga_flag(inside);ga_end();
}
