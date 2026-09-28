// GeomTools oracle, part 1: builders of the Geom, Geom2d and eval geometry the
// write cases print, from a recursive numeric spec (tools/tkgeombase_corpus_geomtools.py
// documents the layout). The Rust twin is crates/tkgeombase/examples/gb_probe/geom_tools/build.rs.
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
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
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <GeomEval_CircularHelixCurve.hxx>
#include <GeomEval_EllipsoidSurface.hxx>
#include <Geom2dEval_SineWaveCurve.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
// A frame with an optional indirect flag: `ax3 indirect`, YReverse when set.
gp_Ax3 gt_ax3(GbReader& r){gp_Ax3 a=r.ax3();if(r.flag())a.YReverse();return a;}
template<class P> NCollection_Array1<P> gt_points(GbReader& r,int n,P(GbReader::*read)()){
 NCollection_Array1<P> p(1,n);for(int i=1;i<=n;++i)p(i)=(r.*read)();return p;}
NCollection_Array1<double> gt_reals(GbReader& r,int n){
 NCollection_Array1<double> v(1,n);for(int i=1;i<=n;++i)v(i)=r.real();return v;}
NCollection_Array1<int> gt_ints(GbReader& r,int n){
 NCollection_Array1<int> v(1,n);for(int i=1;i<=n;++i)v(i)=r.integer();return v;}
// `kind ...`: 1 line, 2 circle, 3 ellipse, 4 parabola, 5 hyperbola, 6 Bezier, 7 B-spline,
// 8 trimmed, 9 offset, 10 GeomEval helix (a class GeomTools does not know).
occ::handle<Geom_Curve> gt_curve(GbReader& r){
 switch(r.integer()){
  case 1:{gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
  case 2:{gp_Ax2 a=r.ax2();return new Geom_Circle(a,r.real());}
  case 3:{gp_Ax2 a=r.ax2();double m=r.real();return new Geom_Ellipse(a,m,r.real());}
  case 4:{gp_Ax2 a=r.ax2();return new Geom_Parabola(a,r.real());}
  case 5:{gp_Ax2 a=r.ax2();double m=r.real();return new Geom_Hyperbola(a,m,r.real());}
  case 6:{bool rat=r.flag();int n=r.integer();auto p=gt_points<gp_Pnt>(r,n,&GbReader::pnt);
   if(rat)return new Geom_BezierCurve(p,gt_reals(r,n));return new Geom_BezierCurve(p);}
  case 7:{bool rat=r.flag(),per=r.flag();int deg=r.integer(),n=r.integer();
   auto p=gt_points<gp_Pnt>(r,n,&GbReader::pnt);NCollection_Array1<double> w(1,n);w.Init(1.);if(rat)w=gt_reals(r,n);
   int k=r.integer();auto kn=gt_reals(r,k);auto m=gt_ints(r,k);
   if(rat)return new Geom_BSplineCurve(p,w,kn,m,deg,per);return new Geom_BSplineCurve(p,kn,m,deg,per);}
  case 8:{double u1=r.real(),u2=r.real();bool s=r.flag();occ::handle<Geom_Curve> b=gt_curve(r);return new Geom_TrimmedCurve(b,u1,u2,s);}
  case 9:{double o=r.real();gp_Dir d=r.dir();occ::handle<Geom_Curve> b=gt_curve(r);return new Geom_OffsetCurve(b,o,d);}
  case 10:{gp_Ax2 a=r.ax2();double rad=r.real();return new GeomEval_CircularHelixCurve(a,rad,r.real());}
 }
 return {};
}
// The same kinds for Geom2d; 10 is the Geom2dEval sine wave.
occ::handle<Geom2d_Curve> gt_curve2d(GbReader& r){
 switch(r.integer()){
  case 1:{gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,r.dir2d());}
  case 2:{gp_Ax22d a=r.ax22d();return new Geom2d_Circle(a,r.real());}
  case 3:{gp_Ax22d a=r.ax22d();double m=r.real();return new Geom2d_Ellipse(a,m,r.real());}
  case 4:{gp_Ax22d a=r.ax22d();return new Geom2d_Parabola(a,r.real());}
  case 5:{gp_Ax22d a=r.ax22d();double m=r.real();return new Geom2d_Hyperbola(a,m,r.real());}
  case 6:{bool rat=r.flag();int n=r.integer();auto p=gt_points<gp_Pnt2d>(r,n,&GbReader::pnt2d);
   if(rat)return new Geom2d_BezierCurve(p,gt_reals(r,n));return new Geom2d_BezierCurve(p);}
  case 7:{bool rat=r.flag(),per=r.flag();int deg=r.integer(),n=r.integer();
   auto p=gt_points<gp_Pnt2d>(r,n,&GbReader::pnt2d);NCollection_Array1<double> w(1,n);w.Init(1.);if(rat)w=gt_reals(r,n);
   int k=r.integer();auto kn=gt_reals(r,k);auto m=gt_ints(r,k);
   if(rat)return new Geom2d_BSplineCurve(p,w,kn,m,deg,per);return new Geom2d_BSplineCurve(p,kn,m,deg,per);}
  case 8:{double u1=r.real(),u2=r.real();bool s=r.flag();occ::handle<Geom2d_Curve> b=gt_curve2d(r);return new Geom2d_TrimmedCurve(b,u1,u2,s);}
  case 9:{double o=r.real();occ::handle<Geom2d_Curve> b=gt_curve2d(r);return new Geom2d_OffsetCurve(b,o);}
  case 10:{gp_Ax2d a=r.ax2d();double amp=r.real(),om=r.real();return new Geom2dEval_SineWaveCurve(a,amp,om,r.real());}
 }
 return {};
}
NCollection_Array2<gp_Pnt> gt_grid(GbReader& r,int nu,int nv){
 NCollection_Array2<gp_Pnt> p(1,nu,1,nv);for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)p(i,j)=r.pnt();return p;}
NCollection_Array2<double> gt_weight_grid(GbReader& r,int nu,int nv){
 NCollection_Array2<double> w(1,nu,1,nv);for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)w(i,j)=r.real();return w;}
// `kind ...`: 1 plane, 2 cylinder, 3 cone, 4 sphere, 5 torus, 6 extrusion, 7 revolution,
// 8 Bezier, 9 B-spline, 10 rectangular trimmed, 11 offset, 12 GeomEval ellipsoid.
occ::handle<Geom_Surface> gt_surface(GbReader& r){
 switch(r.integer()){
  case 1:return new Geom_Plane(gt_ax3(r));
  case 2:{gp_Ax3 a=gt_ax3(r);return new Geom_CylindricalSurface(a,r.real());}
  case 3:{gp_Ax3 a=gt_ax3(r);double ang=r.real();return new Geom_ConicalSurface(a,ang,r.real());}
  case 4:{gp_Ax3 a=gt_ax3(r);return new Geom_SphericalSurface(a,r.real());}
  case 5:{gp_Ax3 a=gt_ax3(r);double m=r.real();return new Geom_ToroidalSurface(a,m,r.real());}
  case 6:{gp_Dir d=r.dir();occ::handle<Geom_Curve> c=gt_curve(r);return new Geom_SurfaceOfLinearExtrusion(c,d);}
  case 7:{gp_Ax1 a=r.ax1();occ::handle<Geom_Curve> c=gt_curve(r);return new Geom_SurfaceOfRevolution(c,a);}
  case 8:{bool rat=r.flag();int nu=r.integer(),nv=r.integer();auto p=gt_grid(r,nu,nv);
   if(rat)return new Geom_BezierSurface(p,gt_weight_grid(r,nu,nv));return new Geom_BezierSurface(p);}
  case 9:{bool rat=r.flag(),up=r.flag(),vp=r.flag();int ud=r.integer(),vd=r.integer(),nu=r.integer(),nv=r.integer();
   auto p=gt_grid(r,nu,nv);NCollection_Array2<double> w(1,nu,1,nv);w.Init(1.);if(rat)w=gt_weight_grid(r,nu,nv);
   int ku=r.integer();auto uk=gt_reals(r,ku);auto um=gt_ints(r,ku);
   int kv=r.integer();auto vk=gt_reals(r,kv);auto vm=gt_ints(r,kv);
   if(rat)return new Geom_BSplineSurface(p,w,uk,vk,um,vm,ud,vd,up,vp);
   return new Geom_BSplineSurface(p,uk,vk,um,vm,ud,vd,up,vp);}
  case 10:{double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();occ::handle<Geom_Surface> b=gt_surface(r);
   return new Geom_RectangularTrimmedSurface(b,u1,u2,v1,v2);}
  case 11:{double o=r.real();occ::handle<Geom_Surface> b=gt_surface(r);return new Geom_OffsetSurface(b,o);}
  case 12:{gp_Ax3 a=gt_ax3(r);double x=r.real(),y=r.real();return new GeomEval_EllipsoidSurface(a,x,y,r.real());}
 }
 return {};
}
