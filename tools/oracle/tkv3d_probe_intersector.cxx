// SelectMgr_BaseIntersector at OCCT 3d097a0; twin: v3_probe/intersector.rs.
#include <SelectMgr_RectangularFrustum.hxx>
#include <SelectMgr_BaseIntersector.hxx>
#include <Graphic3d_Camera.hxx>
void v3_ray_sphere_case(std::istream& input) {
  GaReader r{input};SelectMgr_RectangularFrustum volume;
  gp_Pnt center=r.pnt();double radius=r.real();gp_Pnt origin=r.pnt();gp_Dir direction=r.dir();
  double enter=123.,leave=456.;
  ga_flag(volume.RaySphereIntersection(center,radius,origin,direction,enter,leave));
  ga_real(enter);ga_real(leave);ga_end();
}
void v3_ray_cylinder_case(std::istream& input) {
  GaReader r{input};SelectMgr_RectangularFrustum volume;
  double bottom=r.real(),top=r.real(),height=r.real();gp_Pnt origin=r.pnt();gp_Dir direction=r.dir();
  bool hollow=r.flag();double enter=123.,leave=456.;
  GA_TRY(ga_flag(volume.RayCylinderIntersection(bottom,top,height,origin,direction,hollow,enter,leave));ga_real(enter);ga_real(leave))
  ga_end();
}
void v3_ray_circle_case(std::istream& input) {
  GaReader r{input};SelectMgr_RectangularFrustum volume;
  double radius=r.real();gp_Pnt origin=r.pnt();gp_Dir direction=r.dir();bool filled=r.flag();
  double time=123.;ga_flag(volume.RayCircleIntersection(radius,origin,direction,filled,time));
  ga_real(time);ga_end();
}
void v3_base_intersector_case(std::istream& input) {
  GaReader r{input};SelectMgr_RectangularFrustum volume;
  // Qualify calls to exercise base bodies even where the concrete frustum overrides them.
  volume.SelectMgr_BaseIntersector::SetPixelTolerance(4);
  volume.SelectMgr_BaseIntersector::SetWindowSize(640,480);
  volume.SelectMgr_BaseIntersector::SetViewport(0,0,1,1);
  int width=123,height=456;volume.SelectMgr_BaseIntersector::WindowSize(width,height);
  ga_int(width);ga_int(height);ga_int(volume.GetSelectionType());ga_flag(volume.Camera().IsNull());
  ga_pnt(volume.SelectMgr_BaseIntersector::GetNearPnt());
  ga_pnt(volume.SelectMgr_BaseIntersector::GetFarPnt());
  auto direction=volume.SelectMgr_BaseIntersector::GetViewRayDirection();ga_real(direction.X());ga_real(direction.Y());ga_real(direction.Z());
  auto mouse=volume.SelectMgr_BaseIntersector::GetMousePosition();ga_real(mouse.X());ga_real(mouse.Y());
  ga_real(volume.SelectMgr_BaseIntersector::DistToGeometryCenter(gp_Pnt(1,2,3)));
  ga_pnt(volume.SelectMgr_BaseIntersector::DetectedPoint(10.));
  NCollection_DynamicArray<NCollection_Vec4<double>> planes;planes.Append(NCollection_Vec4<double>(1,2,3,4));
  volume.SelectMgr_BaseIntersector::GetPlanes(planes);ga_int(planes.Size());
  occ::handle<Graphic3d_Camera> camera=new Graphic3d_Camera;
  camera->SetProjectionType(r.flag()?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
  volume.SelectMgr_BaseIntersector::SetCamera(camera);ga_flag(volume.Camera()==camera);ga_flag(volume.Camera()->IsOrthographic());
  volume.SelectMgr_BaseIntersector::SetCamera(nullptr);ga_flag(volume.Camera().IsNull());
  ga_captured([&]{volume.SelectMgr_BaseIntersector::DumpJson(std::cout,-1);});ga_end();
}
