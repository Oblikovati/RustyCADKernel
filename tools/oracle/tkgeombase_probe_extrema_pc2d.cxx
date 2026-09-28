// TKGeomBase oracle: the ExtremaPC2d package and the GCPnts_TangentialDeflection and
// Extrema_Curve2dTool/Extrema_CurveTool::DeflCurvIntervals functions its curve-aware
// partition calls (tkgeombase_probe.cxx). Twin: crates/tkgeombase/examples/gb_probe/pc2d.rs.
//
// A 2D curve specification is read recursively, as in tools/oracle/tkg2d_probe_common.cxx:
// 0 line P D | 1 circle P XD sense R | 2 ellipse P XD sense A B | 3 hyperbola P XD sense A B |
// 4 parabola P XD sense F | 5 bezier rational n (x y w)*n | 6 bspline rational periodic degree
// nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d notcheck <curve> |
// 9 circle on a gp_Ax22d P XD YD R | 14 sine wave P D amplitude omega phase.
// A query is `x y tol mode endpoints`; a result prints status, NbExt, IsDone, IsInfinite,
// InfiniteSquareDistance, MinIndex and MaxIndex (-1 for INVALID_INDEX), then per extremum
// Parameter, Point, SquareDistance, IsMinimum and IsMaximum.
#include <ExtremaPC2d_Curve.hxx>
#include <ExtremaPC2d_DistanceFunction.hxx>
#include <ExtremaPC2d_GridEvaluator.hxx>
#include <Extrema_Curve2dTool.hxx>
#include <Extrema_CurveTool.hxx>
#include <GCPnts_TangentialDeflection.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2dEval_SineWaveCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <NCollection_HArray1.hxx>
#include <Standard_ProgramError.hxx>
occ::handle<Geom2d_Curve> gb_curve2d(GbReader& r);
occ::handle<Geom2d_Curve> gb_bezier2d(GbReader& r){
 bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);
 for(int i=1;i<=n;++i){p(i)=r.pnt2d();w(i)=r.real();}
 if(rational)return new Geom2d_BezierCurve(p,w);return new Geom2d_BezierCurve(p);
}
occ::handle<Geom2d_Curve> gb_bspline2d(GbReader& r){
 bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
 for(int i=1;i<=np;++i){p(i)=r.pnt2d();w(i)=r.real();}
 if(rational)return new Geom2d_BSplineCurve(p,w,k,m,degree,periodic);return new Geom2d_BSplineCurve(p,k,m,degree,periodic);
}
occ::handle<Geom2d_Curve> gb_curve2d(GbReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,r.dir2d());}
 if(kind>=1&&kind<=4){gp_Ax2d a=r.ax2d();bool sense=r.flag();double x=r.real();
  if(kind==1)return new Geom2d_Circle(a,x,sense);
  if(kind==4)return new Geom2d_Parabola(a,x,sense);
  double y=r.real();if(kind==2)return new Geom2d_Ellipse(a,x,y,sense);return new Geom2d_Hyperbola(a,x,y,sense);}
 if(kind==5)return gb_bezier2d(r);
 if(kind==6)return gb_bspline2d(r);
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom2d_Curve> c=gb_curve2d(r);return new Geom2d_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();bool nocheck=r.flag();occ::handle<Geom2d_Curve> c=gb_curve2d(r);return new Geom2d_OffsetCurve(c,d,nocheck);}
 if(kind==9){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();gp_Dir2d y=r.dir2d();return new Geom2d_Circle(gp_Ax22d(p,x,y),r.real());}
 if(kind==14){gp_Ax2d a=r.ax2d();double x=r.real(),y=r.real();return new Geom2dEval_SineWaveCurve(a,x,y,r.real());}
 throw Standard_ProgramError("unknown curve kind");
}
// The result printer, shared with the 3D requests through the point printer.
template<class R,class P> void gb_pc_result(const R& res,P point){
 gb_int(int(res.Status));gb_int(int(res.NbExt()));gb_flag(res.IsDone());gb_flag(res.IsInfinite());
 gb_real(res.InfiniteSquareDistance);
 gb_int(res.MinIndex()==R::INVALID_INDEX?-1:int(res.MinIndex()));
 gb_int(res.MaxIndex()==R::INVALID_INDEX?-1:int(res.MaxIndex()));
 gb_real(res.MinSquareDistance());gb_real(res.MaxSquareDistance());
 for(size_t i=0;i<res.NbExt();++i){gb_real(res[i].Parameter);point(res[i].Point);gb_real(res[i].SquareDistance);gb_flag(res[i].IsMinimum);gb_flag(res[i].IsMaximum);}
}
ExtremaPC2d::SearchMode gb_mode2d(int m){return m==1?ExtremaPC2d::SearchMode::Min:(m==2?ExtremaPC2d::SearchMode::Max:ExtremaPC2d::SearchMode::MinMax);}
// The queries of one evaluator, run in sequence on the same object.
template<class E> void gb_pc2d_queries(E& e,GbReader& r){
 int n=r.integer();
 for(int q=0;q<n;++q){gp_Pnt2d p=r.pnt2d();double tol=r.real();int mode=r.integer();bool ends=r.flag();
  GB_TRY(gb_pc_result(ends?e.PerformWithEndpoints(p,tol,gb_mode2d(mode)):e.Perform(p,tol,gb_mode2d(mode)),gb_pnt2d));}
}
// pc2d_curve: <curve> ctor [u1 u2] queries; ctor 0 handle, 1 handle+range, 2 adaptor, 3 adaptor+range.
void gb_pc2d_curve_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom2d_Curve> c=gb_curve2d(r);int ctor=r.integer();double u1=0,u2=0;if(ctor%2==1){u1=r.real();u2=r.real();}
  Geom2dAdaptor_Curve adaptor;if(ctor>=2)adaptor.Load(c);
  std::unique_ptr<ExtremaPC2d_Curve> e;
  if(ctor==0)e.reset(new ExtremaPC2d_Curve(c));
  if(ctor==1)e.reset(new ExtremaPC2d_Curve(c,u1,u2));
  if(ctor==2)e.reset(new ExtremaPC2d_Curve(adaptor));
  if(ctor==3)e.reset(new ExtremaPC2d_Curve(adaptor,u1,u2));
  gb_flag(e->IsInitialized());
  gb_pc2d_queries(*e,r);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc2d_grid: <curve> n params... dmin dmax queries (no endpoints flag used).
void gb_pc2d_grid_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom2d_Curve> c=gb_curve2d(r);Geom2dAdaptor_Curve adaptor(c);
  std::vector<double> v=r.reals();math_Vector params(v.size());for(size_t i=0;i<v.size();++i)params.ChangeAt(i)=v[i];
  ExtremaPC2d::Domain1D d(r.real(),0.0);d.Max=r.real();
  ExtremaPC2d_GridEvaluator e;e.SetParams(params);
  int n=r.integer();
  for(int q=0;q<n;++q){gp_Pnt2d p=r.pnt2d();double tol=r.real();int mode=r.integer();r.flag();
   GB_TRY(gb_pc_result(e.Perform(adaptor,p,d,tol,gb_mode2d(mode)),gb_pnt2d));}
  gb_flag(ExtremaPC2d_GridEvaluator::IsClosedDomain(adaptor,d));
  GB_TRY(math_Vector a=ExtremaPC2d_GridEvaluator::BuildCurveAwareParams(adaptor,d);gb_int(int(a.Size()));for(size_t i=0;i<a.Size();++i)gb_real(a.At(i)));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc2d_uniform: umin umax n.
void gb_pc2d_uniform_case(std::istream& in){
 GbReader r{in};double a=r.real(),b=r.real();int n=r.integer();
 GB_TRY(math_Vector v=ExtremaPC2d_GridEvaluator::BuildUniformParams(a,b,size_t(n));gb_int(int(v.Size()));for(size_t i=0;i<v.Size();++i)gb_real(v.At(i)));
 gb_end();
}
// pc2d_distance: <curve> x y n u...: Value, Values and Derivative of the distance function.
void gb_pc2d_distance_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom2d_Curve> c=gb_curve2d(r);Geom2dAdaptor_Curve adaptor(c);gp_Pnt2d p=r.pnt2d();
  ExtremaPC2d_DistanceFunction f(adaptor,p);std::vector<double> u=r.reals();
  gb_real(f.FirstParameter());gb_real(f.LastParameter());gb_pnt2d(f.Point());
  for(double x:u){double v=0,dv=0,d=0;GB_TRY(f.Value(x,v);gb_real(v);f.Values(x,v,dv);gb_real(v);gb_real(dv);f.Derivative(x,d);gb_real(d));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// pc2d_endpoints: <curve> dmin dmax x y mode closed: AddEndpointExtrema on the adaptor.
void gb_pc2d_endpoints_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom2d_Curve> c=gb_curve2d(r);Geom2dAdaptor_Curve adaptor(c);
  ExtremaPC2d::Domain1D d(r.real(),0.0);d.Max=r.real();gp_Pnt2d p=r.pnt2d();int mode=r.integer();bool closed=r.flag();
  ExtremaPC2d::Result res;ExtremaPC2d::AddEndpointExtrema(res,p,d,adaptor,gb_mode2d(mode),closed);gb_pc_result(res,gb_pnt2d);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// gcpnts_td2d: <curve> ranged [u1 u2] angular curvature minpoints utol minlen, then
// Extrema_Curve2dTool::DeflCurvIntervals of the adaptor.
void gb_gcpnts_td2d_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Geom2d_Curve> c=gb_curve2d(r);Geom2dAdaptor_Curve adaptor(c);bool ranged=r.flag();double u1=0,u2=0;if(ranged){u1=r.real();u2=r.real();}
  double a=r.real(),cd=r.real();int n=r.integer();double ut=r.real(),ml=r.real();
  GB_TRY(GCPnts_TangentialDeflection t;if(ranged)t.Initialize(adaptor,u1,u2,a,cd,n,ut,ml);else t.Initialize(adaptor,a,cd,n,ut,ml);
   gb_int(t.NbPoints());for(int i=1;i<=t.NbPoints();++i){gb_real(t.Parameter(i));gb_pnt(t.Value(i));});
  GB_TRY(occ::handle<NCollection_HArray1<double>> h=Extrema_Curve2dTool::DeflCurvIntervals(adaptor);gb_int(h->Length());for(double x:h->Array1())gb_real(x));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
