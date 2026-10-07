// Graphic3d_Camera, the Rust twin of crates/tkservice/examples/sv_probe/camera.rs.
#include <Bnd_Box.hxx>
#include <Graphic3d_Camera.hxx>
#include <gp_Pln.hxx>
template <typename T> void sv_mat4(const NCollection_Mat4<T>& m){for(int r=0;r<4;++r)for(int c=0;c<4;++c)ga_real(m.GetValue(r,c));}
// Reads `Proj ZeroToOne Eye(3) Center(3) Up(3) Axial(3) FOVy Aspect ZNear ZFar Scale IODType IOD
// FocusType Focus Tile(7)` and applies the setters in the twin's order; raised setters print
// their exception type.
occ::handle<Graphic3d_Camera> sv_camera_read(GaReader& r){
 occ::handle<Graphic3d_Camera> cam=new Graphic3d_Camera();
 int proj=r.integer();bool zeroToOne=r.flag();gp_Pnt eye=r.pnt(),center=r.pnt();gp_XYZ up=r.xyz(),axial=r.xyz();
 double fovy=r.real(),aspect=r.real(),zn=r.real(),zf=r.real(),scale=r.real();int iodType=r.integer();double iod=r.real();
 int focusType=r.integer();double focus=r.real();int tile[7];for(int& t:tile)t=r.integer();
 cam->SetProjectionType((Graphic3d_Camera::Projection)proj);cam->SetZeroToOneDepth(zeroToOne);
 GA_TRY(cam->SetEyeAndCenter(eye,center))GA_TRY(cam->SetUp(gp_Dir(up)))GA_TRY(cam->SetAxialScale(axial))
 cam->SetFOVy(fovy);cam->SetAspect(aspect);GA_TRY(cam->SetZRange(zn,zf))cam->SetScale(scale);
 cam->SetIOD((Graphic3d_Camera::IODType)iodType,iod);cam->SetZFocus((Graphic3d_Camera::FocusType)focusType,focus);
 Graphic3d_CameraTile aTile;aTile.TotalSize.SetValues(tile[0],tile[1]);aTile.TileSize.SetValues(tile[2],tile[3]);
 aTile.Offset.SetValues(tile[4],tile[5]);aTile.IsTopDown=tile[6]!=0;cam->SetTile(aTile);
 return cam;}
// A camera: matrices in double and float, stereo projections, conversions of a point,
// frustum planes and points, view dimensions and the derived values.
void sv_camera_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_Camera> cam=sv_camera_read(r);gp_Pnt p=r.pnt();
 sv_mat4(cam->OrientationMatrix());sv_mat4(cam->ProjectionMatrix());sv_mat4(cam->ProjectionStereoLeft());sv_mat4(cam->ProjectionStereoRight());
 sv_mat4(cam->OrientationMatrixF());sv_mat4(cam->ProjectionMatrixF());sv_mat4(cam->ProjectionStereoLeftF());sv_mat4(cam->ProjectionStereoRightF());
 NCollection_Mat4<double> pl,hl,pr,hr;cam->StereoProjection(pl,hl,pr,hr);sv_mat4(pl);sv_mat4(hl);sv_mat4(pr);sv_mat4(hr);
 ga_pnt(cam->Project(p));ga_pnt(cam->UnProject(p));ga_pnt(cam->ConvertWorld2View(p));ga_pnt(cam->ConvertView2World(p));
 ga_pnt(cam->ConvertView2Proj(p));ga_pnt(cam->ConvertProj2View(p));
 GA_TRY(gp_Pln pls[6];cam->Frustum(pls[0],pls[1],pls[2],pls[3],pls[4],pls[5]);
  for(const gp_Pln& pln:pls){double a,b,c,d;pln.Coefficients(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);})
 NCollection_Array1<NCollection_Vec3<double>> pts(0,Graphic3d_Camera::FrustumVerticesNB-1);cam->FrustumPoints(pts);
 for(int i=0;i<Graphic3d_Camera::FrustumVerticesNB;++i){ga_real(pts[i].x());ga_real(pts[i].y());ga_real(pts[i].z());}
 ga_xyz(cam->ViewDimensions());ga_real(cam->Scale());ga_real(cam->Distance());ga_real(cam->FOVx());ga_real(cam->NDC2dOffsetX());ga_real(cam->NDC2dOffsetY());
 ga_pnt(cam->Eye());ga_pnt(cam->Center());ga_dir(cam->Direction());ga_dir(cam->Up());
 GA_TRY(ga_dir(cam->OrthogonalizedUp()))
 ga_end();}
// A camera and two boxes `Min(3) Max(3)`, then `Resolution ToEnlarge ScaleFactor`: ZFitAll,
// FitMinMax and the resulting camera.
void sv_camera_fit_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_Camera> cam=sv_camera_read(r);
 Bnd_Box model,graphic;gp_Pnt m0=r.pnt(),m1=r.pnt(),g0=r.pnt(),g1=r.pnt();model.Add(m0);model.Add(m1);graphic.Add(g0);graphic.Add(g1);
 double resolution=r.real();bool enlarge=r.flag();double factor=r.real();
 GA_TRY(double zn=0,zf=0;ga_flag(cam->ZFitAll(factor,model,graphic,zn,zf));ga_real(zn);ga_real(zf))
 GA_TRY(ga_flag(cam->FitMinMax(graphic,resolution,enlarge)))
 ga_pnt(cam->Eye());ga_pnt(cam->Center());ga_real(cam->Scale());ga_real(cam->Distance());
 ga_end();}
// Two cameras and `T`: Interpolate into a third camera.
void sv_camera_interp_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_Camera> a=sv_camera_read(r),b=sv_camera_read(r);double t=r.real();
 occ::handle<Graphic3d_Camera> c=new Graphic3d_Camera();
 GA_TRY(Graphic3d_Camera::Interpolate(a,b,t,c);ga_pnt(c->Eye());ga_pnt(c->Center());ga_dir(c->Up());ga_dir(c->Direction());ga_real(c->Scale());ga_real(c->Distance()))
 ga_end();}
