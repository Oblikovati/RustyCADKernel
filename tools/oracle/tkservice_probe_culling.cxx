// Graphic3d_CullingTool, the Rust twin of crates/tkservice/examples/sv_probe/culling.rs.
#include <Graphic3d_CullingTool.hxx>
// A camera (as sv_camera), then `Tx Ty Tz Width Height Ratio Distance Size N (Min(3) Max(3))*`:
// the view volume (a model translation when not zero), the culling context and each box's
// frustum, distance and size tests with the "fully inside" flags.
void sv_culling_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_Camera> cam=sv_camera_read(r);gp_XYZ t=r.xyz();
 int w=r.integer(),h=r.integer();double ratio=r.real(),dist=r.real(),size=r.real();
 NCollection_Mat4<double> model;model.SetValue(0,3,t.X());model.SetValue(1,3,t.Y());model.SetValue(2,3,t.Z());
 Graphic3d_CullingTool tool;tool.SetViewVolume(cam,model);tool.SetViewportSize(w,h,ratio);tool.CacheClipPtsProjections();
 Graphic3d_CullingTool::CullingContext ctx;tool.SetCullingDistance(ctx,dist);tool.SetCullingSize(ctx,size);
 ga_real(ctx.DistCull);ga_real(ctx.SizeCull2);ga_int(tool.ViewportWidth());ga_int(tool.ViewportHeight());
 ga_real(tool.CameraEye().x());ga_real(tool.CameraEye().y());ga_real(tool.CameraEye().z());
 ga_real(tool.CameraDirection().x());ga_real(tool.CameraDirection().y());ga_real(tool.CameraDirection().z());
 ga_real(tool.SignedPlanePointDistance(NCollection_Vec4<double>(1,2,2,-3),NCollection_Vec4<double>(t.X(),t.Y(),t.Z(),1)));
 int n=r.integer();
 for(int i=0;i<n;++i){gp_XYZ a=r.xyz(),b=r.xyz();NCollection_Vec3<double> mn(a.X(),a.Y(),a.Z()),mx(b.X(),b.Y(),b.Z());
  ga_flag(tool.IsOutFrustum(mn,mx));bool in1=true;ga_flag(tool.IsOutFrustum(mn,mx,&in1));ga_flag(in1);
  bool in2=true;ga_flag(tool.IsTooDistant(ctx,mn,mx,&in2));ga_flag(in2);ga_flag(tool.IsTooSmall(ctx,mn,mx));
  bool in3=true;ga_flag(tool.IsCulled(ctx,mn,mx,&in3));ga_flag(in3);ga_flag(tool.IsCulled(ctx,mn,mx));}
 ga_end();}
