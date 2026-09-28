// TKGeomBase oracle: the ExtremaPC package (tkgeombase_probe.cxx), and
// GCPnts_TangentialDeflection and Extrema_CurveTool::DeflCurvIntervals on 3D curves.
// Twin: crates/tkgeombase/examples/gb_probe/pc3d.rs.
//
// A 3D curve specification is read recursively:
// 0 line P D | 1 circle P N X R | 2 ellipse P N X A B | 3 hyperbola P N X A B |
// 4 parabola P N X F | 5 bezier rational n (x y z w)*n | 6 bspline rational periodic degree
// nk (k m)*nk np (x y z w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d D <curve>.
// A transformation is `kind a b c d e f g`: 0 identity, 1 translation (a b c),
// 2 rotation about the axis (a b c)(d e f) by g, 3 scale about (a b c) by d, 4 the
// rotation 2 then the scale about the origin by (a).
// Queries and results are printed as in tkgeombase_probe_extrema_pc2d.cxx.
#include <ExtremaPC_Curve.hxx>
#include <ExtremaPC_DistanceFunction.hxx>
#include <ExtremaPC_GridEvaluator.hxx>
#include <ExtremaPC_Planar.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_TransformedCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_TrimmedCurve.hxx>
occ::handle<Geom_Curve> gb_curve3d(GbReader& r);
occ::handle<Geom_Curve> gb_bezier3d(GbReader& r){
 bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> w(1,n);
 for(int i=1;i<=n;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom_BezierCurve(p,w);return new Geom_BezierCurve(p);
}
occ::handle<Geom_Curve> gb_bspline3d(GbReader& r){
 bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 int np=r.integer();NCollection_Array1<gp_Pnt> p(1,np);NCollection_Array1<double> w(1,np);
 for(int i=1;i<=np;++i){p(i)=r.pnt();w(i)=r.real();}
 if(rational)return new Geom_BSplineCurve(p,w,k,m,degree,periodic);return new Geom_BSplineCurve(p,k,m,degree,periodic);
}
occ::handle<Geom_Curve> gb_curve3d(GbReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind>=1&&kind<=4){gp_Ax2 a=r.ax2();double x=r.real();
  if(kind==1)return new Geom_Circle(a,x);
  if(kind==4)return new Geom_Parabola(a,x);
  double y=r.real();if(kind==2)return new Geom_Ellipse(a,x,y);return new Geom_Hyperbola(a,x,y);}
 if(kind==5)return gb_bezier3d(r);
 if(kind==6)return gb_bspline3d(r);
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom_Curve> c=gb_curve3d(r);return new Geom_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();gp_Dir v=r.dir();occ::handle<Geom_Curve> c=gb_curve3d(r);return new Geom_OffsetCurve(c,d,v);}
 throw Standard_ProgramError("unknown curve kind");
}
gp_Trsf gb_trsf3d(GbReader& r){
 int kind=r.integer();double p[7];for(double& v:p)v=r.real();gp_Trsf t;
 if(kind==1)t.SetTranslation(gp_Vec(p[0],p[1],p[2]));
 if(kind==2||kind==4)t.SetRotation(gp_Ax1(gp_Pnt(p[0],p[1],p[2]),gp_Dir(p[3],p[4],p[5])),p[6]);
 if(kind==3)t.SetScale(gp_Pnt(p[0],p[1],p[2]),p[3]);
 if(kind==4){gp_Trsf s;s.SetScale(gp_Pnt(),p[0]);t=t.Multiplied(s);}
 return t;
}
ExtremaPC::SearchMode gb_mode3d(int m){return m==1?ExtremaPC::SearchMode::Min:(m==2?ExtremaPC::SearchMode::Max:ExtremaPC::SearchMode::MinMax);}
template<class E> void gb_pc3d_queries(E& e,GbReader& r){
 int n=r.integer();
 for(int q=0;q<n;++q){gp_Pnt p=r.pnt();double tol=r.real();int mode=r.integer();bool ends=r.flag();
  GB_TRY(gb_pc_result(ends?e.PerformWithEndpoints(p,tol,gb_mode3d(mode)):e.Perform(p,tol,gb_mode3d(mode)),gb_pnt));}
}
// pc_curve: <curve> ctor [trsf] [u1 u2] queries; ctor 0 handle, 1 handle+range, 2 adaptor,
// 3 adaptor+range, 4 transformed, 5 transformed+range.
void gb_pc_curve_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);int ctor=r.integer();gp_Trsf t;if(ctor>=4)t=gb_trsf3d(r);
  double u1=0,u2=0;if(ctor%2==1){u1=r.real();u2=r.real();}
  GeomAdaptor_Curve adaptor;if(ctor==2||ctor==3)adaptor.Load(c);
  std::unique_ptr<GeomAdaptor_TransformedCurve> transformed;if(ctor>=4)transformed.reset(new GeomAdaptor_TransformedCurve(c,t));
  std::unique_ptr<ExtremaPC_Curve> e;
  if(ctor==0)e.reset(new ExtremaPC_Curve(c));
  if(ctor==1)e.reset(new ExtremaPC_Curve(c,u1,u2));
  if(ctor==2)e.reset(new ExtremaPC_Curve(adaptor));
  if(ctor==3)e.reset(new ExtremaPC_Curve(adaptor,u1,u2));
  if(ctor==4)e.reset(new ExtremaPC_Curve(*transformed));
  if(ctor==5)e.reset(new ExtremaPC_Curve(*transformed,u1,u2));
  gb_flag(e->IsInitialized());
  gb_pc3d_queries(*e,r);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc_grid: <curve> n params... dmin dmax queries.
void gb_pc_grid_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);GeomAdaptor_Curve adaptor(c);
  std::vector<double> v=r.reals();math_Vector params(v.size());for(size_t i=0;i<v.size();++i)params.ChangeAt(i)=v[i];
  ExtremaPC::Domain1D d(r.real(),0.0);d.Max=r.real();
  ExtremaPC_GridEvaluator e;e.SetParams(params);
  int n=r.integer();
  for(int q=0;q<n;++q){gp_Pnt p=r.pnt();double tol=r.real();int mode=r.integer();r.flag();
   GB_TRY(gb_pc_result(e.Perform(adaptor,p,d,tol,gb_mode3d(mode)),gb_pnt));}
  gb_flag(ExtremaPC_GridEvaluator::IsClosedDomain(adaptor,d));
  GB_TRY(math_Vector a=ExtremaPC_GridEvaluator::BuildCurveAwareParams(adaptor,d);gb_int(int(a.Size()));for(size_t i=0;i<a.Size();++i)gb_real(a.At(i)));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc_distance: <curve> x y z n u...
void gb_pc_distance_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);GeomAdaptor_Curve adaptor(c);gp_Pnt p=r.pnt();
  ExtremaPC_DistanceFunction f(adaptor,p);std::vector<double> u=r.reals();
  gb_real(f.FirstParameter());gb_real(f.LastParameter());gb_pnt(f.Point());
  for(double x:u){double v=0,dv=0,d=0;GB_TRY(f.Value(x,v);gb_real(v);f.Values(x,v,dv);gb_real(v);gb_real(dv);f.Derivative(x,d);gb_real(d));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc_endpoints: <curve> dmin dmax x y z mode closed.
void gb_pc_endpoints_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);GeomAdaptor_Curve adaptor(c);
  ExtremaPC::Domain1D d(r.real(),0.0);d.Max=r.real();gp_Pnt p=r.pnt();int mode=r.integer();bool closed=r.flag();
  ExtremaPC::Result res;ExtremaPC::AddEndpointExtrema(res,p,d,adaptor,gb_mode3d(mode),closed);gb_pc_result(res,gb_pnt);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc_project: <curve> n u...: LinePlane of a line, ProjectBezier/ProjectBSpline/ProjectOffset
// of the others; the flag, the plane and the 2D curve points at the parameters.
void gb_pc_project_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);std::vector<double> u=r.reals();gp_Pln pl;occ::handle<Geom2d_Curve> c2;bool done=false;
  if(auto l=occ::down_cast<Geom_Line>(c)){GB_TRY(pl=ExtremaPC::LinePlane(l->Lin());done=true);}
  if(auto b=occ::down_cast<Geom_BezierCurve>(c)){occ::handle<Geom2d_BezierCurve> x;GB_TRY(done=ExtremaPC::ProjectBezier(b,pl,x);c2=x);}
  if(auto b=occ::down_cast<Geom_BSplineCurve>(c)){occ::handle<Geom2d_BSplineCurve> x;GB_TRY(done=ExtremaPC::ProjectBSpline(b,pl,x);c2=x);}
  if(auto o=occ::down_cast<Geom_OffsetCurve>(c)){occ::handle<Geom2d_OffsetCurve> x;GB_TRY(done=ExtremaPC::ProjectOffset(o,pl,x);c2=x);}
  gb_flag(done);gb_ax3(pl.Position());gb_flag(c2.IsNull());
  if(!c2.IsNull())for(double x:u)GB_TRY(gb_pnt2d(c2->Value(x)));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// gcpnts_td3d: <curve> ranged [u1 u2] angular curvature minpoints utol minlen.
void gb_gcpnts_td3d_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom_Curve> c=gb_curve3d(r);GeomAdaptor_Curve adaptor(c);bool ranged=r.flag();double u1=0,u2=0;if(ranged){u1=r.real();u2=r.real();}
  double a=r.real(),cd=r.real();int n=r.integer();double ut=r.real(),ml=r.real();
  GB_TRY(GCPnts_TangentialDeflection t;if(ranged)t.Initialize(adaptor,u1,u2,a,cd,n,ut,ml);else t.Initialize(adaptor,a,cd,n,ut,ml);
   gb_int(t.NbPoints());for(int i=1;i<=t.NbPoints();++i){gb_real(t.Parameter(i));gb_pnt(t.Value(i));});
  GB_TRY(occ::handle<NCollection_HArray1<double>> h=Extrema_CurveTool::DeflCurvIntervals(adaptor);gb_int(h->Length());for(double x:h->Array1())gb_real(x));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
