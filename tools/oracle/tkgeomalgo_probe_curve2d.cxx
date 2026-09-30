// Shared native 2D curve reader; format matches ga_probe/curve2d.rs.
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2dInt_Geom2dCurveTool.hxx>
#include <Geom2dInt_IntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheIntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter.hxx>
#include <Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheProjPCurOfGInter.hxx>
#include <Standard_ProgramError.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
gp_Dir2d ga_dir2d(GaReader& r){double x=r.real();return gp_Dir2d(x,r.real());}
gp_Ax2d ga_ax2d(GaReader& r){gp_Pnt2d p=r.pnt2d();return gp_Ax2d(p,ga_dir2d(r));}
occ::handle<Geom2d_Curve> ga_curve2d(GaReader& r);
occ::handle<Geom2d_Curve> ga_bezier2d(GaReader& r){
 bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);
 for(int i=1;i<=n;++i){p(i)=r.pnt2d();w(i)=r.real();}
 if(rational)return new Geom2d_BezierCurve(p,w);return new Geom2d_BezierCurve(p);
}
occ::handle<Geom2d_Curve> ga_bspline2d(GaReader& r){
 bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
 for(int i=1;i<=np;++i){p(i)=r.pnt2d();w(i)=r.real();}
 if(rational)return new Geom2d_BSplineCurve(p,w,k,m,degree,periodic);return new Geom2d_BSplineCurve(p,k,m,degree,periodic);
}
occ::handle<Geom2d_Curve> ga_curve2d(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,ga_dir2d(r));}
 if(kind>=1&&kind<=4){gp_Ax2d a=ga_ax2d(r);bool sense=r.flag();double x=r.real();
  if(kind==1)return new Geom2d_Circle(a,x,sense);
  if(kind==4)return new Geom2d_Parabola(a,x,sense);
  double y=r.real();if(kind==2)return new Geom2d_Ellipse(a,x,y,sense);return new Geom2d_Hyperbola(a,x,y,sense);}
 if(kind==5)return ga_bezier2d(r);
 if(kind==6)return ga_bspline2d(r);
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom2d_Curve> c=ga_curve2d(r);return new Geom2d_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();bool nocheck=r.flag();occ::handle<Geom2d_Curve> c=ga_curve2d(r);return new Geom2d_OffsetCurve(c,d,nocheck);}
 throw Standard_ProgramError("unknown curve kind");
}
