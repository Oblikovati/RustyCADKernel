// Graphic3d_TransformPers and Graphic3d_TransformUtils, the Rust twin of
// crates/tkservice/examples/sv_probe/tpers.rs.
#include <Graphic3d_TransformPers.hxx>
#include <Graphic3d_TransformPersScaledAbove.hxx>
#include <Graphic3d_TransformUtils.hxx>
// A camera (as sv_camera), then `Kind Mode X Y Z Corner OffsetX OffsetY Scale Width Height
// Min(3) Max(3)`: Kind 0 mode-only, 1 anchor point, 2 corner, 3 scaled-above. Prints the
// persistent scale, ComputeApply, Apply in double and float, Compute and the box Apply members.
void sv_tpers_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_Camera> cam=sv_camera_read(r);
 int kind=r.integer(),mode=r.integer();gp_Pnt p=r.pnt();int corner=r.integer(),ox=r.integer(),oy=r.integer();
 double minScale=r.real();int w=r.integer(),h=r.integer();gp_Pnt b0=r.pnt(),b1=r.pnt();
 occ::handle<Graphic3d_TransformPers> tp;
 try{
  if(kind==0)tp=new Graphic3d_TransformPers((Graphic3d_TransModeFlags)mode);
  else if(kind==1)tp=new Graphic3d_TransformPers((Graphic3d_TransModeFlags)mode,p);
  else if(kind==2)tp=new Graphic3d_TransformPers((Graphic3d_TransModeFlags)mode,(Aspect_TypeOfTriedronPosition)corner,NCollection_Vec2<int>(ox,oy));
  else tp=new Graphic3d_TransformPersScaledAbove(minScale,p);
 }catch(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';ga_end();return;}
 ga_int(tp->Mode());GA_TRY(ga_pnt(tp->AnchorPoint()))GA_TRY(ga_int(tp->Corner2d()))
 GA_TRY(NCollection_Vec2<int> o=tp->Offset2d();ga_int(o.x());ga_int(o.y()))
 if(tp->IsZoomOrRotate()||tp->IsAxial())ga_real(tp->persistentScale(cam,w,h));
 GA_TRY(occ::handle<Graphic3d_Camera> c=new Graphic3d_Camera(*cam);sv_mat4(tp->ComputeApply(c,w,h));ga_flag(c->IsOrthographic()))
 GA_TRY(NCollection_Mat4<double> wv=cam->OrientationMatrix();tp->Apply(cam,cam->ProjectionMatrix(),wv,w,h);sv_mat4(wv))
 GA_TRY(NCollection_Mat4<float> wv=cam->OrientationMatrixF();tp->Apply(cam,cam->ProjectionMatrixF(),wv,w,h,&p,true);sv_mat4(wv))
 GA_TRY(sv_mat4(tp->Compute(cam,cam->ProjectionMatrix(),cam->OrientationMatrix(),w,h,true)))
 GA_TRY(Graphic3d_BndBox3d box(NCollection_Vec3<double>(b0.X(),b0.Y(),b0.Z()),NCollection_Vec3<double>(b1.X(),b1.Y(),b1.Z()));
  tp->Apply(cam,cam->ProjectionMatrix(),cam->OrientationMatrix(),w,h,box);
  ga_real(box.CornerMin().x());ga_real(box.CornerMin().y());ga_real(box.CornerMin().z());
  ga_real(box.CornerMax().x());ga_real(box.CornerMax().y());ga_real(box.CornerMax().z()))
 GA_TRY(Bnd_Box bb;bb.Update(b0.X(),b0.Y(),b0.Z(),b1.X(),b1.Y(),b1.Z());
  tp->Apply(cam,cam->ProjectionMatrix(),cam->OrientationMatrix(),w,h,bb);
  double v[6];bb.Get(v[0],v[1],v[2],v[3],v[4],v[5]);for(double x:v)ga_real(x);)
 ga_end();}
// `Kind A X Y Z S0 S1 S2 T0 T1 T2 L R B T N F Px Py Pz V0 V1 V2 V3`, Kind 0 double / 1 float:
// Rotate, Scale, Translate of the identity, ConstructRotate into a fresh matrix, Ortho, Ortho2D,
// Project and UnProject through them, ScaleFactor.
template<class T> void sv_tutils(GaReader& r){
 T a=(T)r.real(),x=(T)r.real(),y=(T)r.real(),z=(T)r.real();T s[3],t[3],o[6],pt[3];int vp[4];
 for(T& v:s)v=(T)r.real();for(T& v:t)v=(T)r.real();for(T& v:o)v=(T)r.real();for(T& v:pt)v=(T)r.real();for(int& v:vp)v=r.integer();
 NCollection_Mat4<T> m;Graphic3d_TransformUtils::Rotate<T>(m,a,x,y,z);sv_mat4(m);
 Graphic3d_TransformUtils::Scale<T>(m,s[0],s[1],s[2]);sv_mat4(m);Graphic3d_TransformUtils::Translate<T>(m,t[0],t[1],t[2]);sv_mat4(m);
 NCollection_Mat4<T> c;Graphic3d_TransformUtils::ConstructRotate<T>(c,a,x,y,z);sv_mat4(c);
 NCollection_Mat4<T> proj;Graphic3d_TransformUtils::Ortho<T>(proj,o[0],o[1],o[2],o[3],o[4],o[5]);sv_mat4(proj);
 NCollection_Mat4<T> proj2;Graphic3d_TransformUtils::Ortho2D<T>(proj2,o[0],o[1],o[2],o[3]);sv_mat4(proj2);
 T wx=0,wy=0,wz=0;bool ok=Graphic3d_TransformUtils::Project<T>(pt[0],pt[1],pt[2],m,proj,vp,wx,wy,wz);ga_flag(ok);
 if(ok){ga_real(wx);ga_real(wy);ga_real(wz);}
 T ux=0,uy=0,uz=0;ok=Graphic3d_TransformUtils::UnProject<T>(wx,wy,wz,m,proj,vp,ux,uy,uz);ga_flag(ok);
 if(ok){ga_real(ux);ga_real(uy);ga_real(uz);}
 ga_real(Graphic3d_TransformUtils::ScaleFactor<T>(m));}
void sv_tutils_case(std::istream& input){
 GaReader r{input};int kind=r.integer();if(kind==0)sv_tutils<double>(r);else sv_tutils<float>(r);
 gp_Trsf tr;tr.SetRotation(gp_Ax1(gp_Pnt(1,2,3),gp_Dir(0,0.6,0.8)),0.7);tr.SetScaleFactor(2.5);
 NCollection_Mat4<double> md;Graphic3d_TransformUtils::Convert<double>(tr,md);sv_mat4(md);
 NCollection_Mat4<float> mf;Graphic3d_TransformUtils::Convert<float>(tr,mf);sv_mat4(mf);
 ga_end();}
