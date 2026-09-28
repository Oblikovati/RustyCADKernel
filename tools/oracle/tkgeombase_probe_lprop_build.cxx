// TKGeomBase oracle: the curve and surface builders shared by the LProp/GeomLProp and
// Hermit probes (tkgeombase_probe_lprop.cxx, tkgeombase_probe_hermit.cxx). A curve or
// surface is read as a kind code followed by its data (tools/tkgeombase_corpus_lprop.py
// writes them); the Rust twin is crates/tkgeombase/examples/gb_probe/lprop/build.rs.
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
// `n` knots with their multiplicities, read as (knot, mult) pairs.
void gb_lp_knots(GbReader& r,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 for(int i=1;i<=k.Length();++i){k(i)=r.real();m(i)=r.integer();}
}
// Kinds: 0 line, 1 circle, 2 ellipse, 3 hyperbola, 4 parabola (gp_Ax2d, sense, radii),
// 5 rational Bezier, 6 rational B-spline, 7 trimmed, 8 offset.
occ::handle<Geom2d_Curve> gb_lp_curve2d(GbReader& r){
 const int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,r.dir2d());}
 if(kind>=1&&kind<=4){
  gp_Ax2d a=r.ax2d();bool s=r.flag();double r1=r.real();
  if(kind==1)return new Geom2d_Circle(gp_Circ2d(a,r1,s));
  if(kind==4)return new Geom2d_Parabola(gp_Parab2d(a,r1,s));
  double r2=r.real();
  if(kind==2)return new Geom2d_Ellipse(gp_Elips2d(a,r1,r2,s));
  return new Geom2d_Hyperbola(gp_Hypr2d(a,r1,r2,s));
 }
 if(kind==5||kind==6){
  int deg=0;bool per=false;
  if(kind==6){deg=r.integer();per=r.flag();}
  const int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
  for(int i=1;i<=np;++i){p(i)=r.pnt2d();w(i)=r.real();}
  if(kind==5)return new Geom2d_BezierCurve(p,w);
  const int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);gb_lp_knots(r,k,m);
  return new Geom2d_BSplineCurve(p,w,k,m,deg,per);
 }
 if(kind==7){double u1=r.real(),u2=r.real();occ::handle<Geom2d_Curve> b=gb_lp_curve2d(r);return new Geom2d_TrimmedCurve(b,u1,u2);}
 double d=r.real();occ::handle<Geom2d_Curve> b=gb_lp_curve2d(r);return new Geom2d_OffsetCurve(b,d);
}
// The 3D kinds of gb_lp_curve2d; conics take a gp_Ax2 (location, normal, X), the
// offset curve its offset and reference direction.
occ::handle<Geom_Curve> gb_lp_curve3d(GbReader& r){
 const int kind=r.integer();
 if(kind==0){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind>=1&&kind<=4){
  gp_Ax2 a=r.ax2();double r1=r.real();
  if(kind==1)return new Geom_Circle(a,r1);
  if(kind==4)return new Geom_Parabola(a,r1);
  double r2=r.real();
  if(kind==2)return new Geom_Ellipse(a,r1,r2);
  return new Geom_Hyperbola(a,r1,r2);
 }
 if(kind==5||kind==6){
  int deg=0;bool per=false;
  if(kind==6){deg=r.integer();per=r.flag();}
  const int np=r.integer();NCollection_Array1<gp_Pnt> p(1,np);NCollection_Array1<double> w(1,np);
  for(int i=1;i<=np;++i){p(i)=r.pnt();w(i)=r.real();}
  if(kind==5)return new Geom_BezierCurve(p,w);
  const int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);gb_lp_knots(r,k,m);
  return new Geom_BSplineCurve(p,w,k,m,deg,per);
 }
 if(kind==7){double u1=r.real(),u2=r.real();occ::handle<Geom_Curve> b=gb_lp_curve3d(r);return new Geom_TrimmedCurve(b,u1,u2);}
 double d=r.real();gp_Dir v=r.dir();occ::handle<Geom_Curve> b=gb_lp_curve3d(r);return new Geom_OffsetCurve(b,d,v);
}
// Kinds: 0 plane, 1 cylinder, 2 cone, 3 sphere, 4 torus (gp_Ax3 then radii/angle),
// 5 rational B-spline surface, 6 revolution, 7 extrusion, 8 offset.
occ::handle<Geom_Surface> gb_lp_surface(GbReader& r){
 const int kind=r.integer();
 if(kind<=4){
  gp_Ax3 a=r.ax3();
  if(kind==0)return new Geom_Plane(a);
  double r1=r.real();
  if(kind==1)return new Geom_CylindricalSurface(a,r1);
  if(kind==3)return new Geom_SphericalSurface(a,r1);
  double r2=r.real();
  if(kind==2)return new Geom_ConicalSurface(a,r1,r2);
  return new Geom_ToroidalSurface(a,r1,r2);
 }
 if(kind==5){
  int du=r.integer(),dv=r.integer(),nu=r.integer(),nv=r.integer();
  NCollection_Array2<gp_Pnt> p(1,nu,1,nv);NCollection_Array2<double> w(1,nu,1,nv);
  for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j){p(i,j)=r.pnt();w(i,j)=r.real();}
  const int nku=r.integer();NCollection_Array1<double> ku(1,nku);NCollection_Array1<int> mu(1,nku);gb_lp_knots(r,ku,mu);
  const int nkv=r.integer();NCollection_Array1<double> kv(1,nkv);NCollection_Array1<int> mv(1,nkv);gb_lp_knots(r,kv,mv);
  return new Geom_BSplineSurface(p,w,ku,kv,mu,mv,du,dv);
 }
 if(kind==6){gp_Ax1 a=r.ax1();occ::handle<Geom_Curve> c=gb_lp_curve3d(r);return new Geom_SurfaceOfRevolution(c,a);}
 if(kind==7){gp_Dir d=r.dir();occ::handle<Geom_Curve> c=gb_lp_curve3d(r);return new Geom_SurfaceOfLinearExtrusion(c,d);}
 double d=r.real();occ::handle<Geom_Surface> s=gb_lp_surface(r);return new Geom_OffsetSurface(s,d);
}
