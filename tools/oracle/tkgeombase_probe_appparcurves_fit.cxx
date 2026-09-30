// The AppParCurves generics through their AppDef instantiations (AppDef_MultiLine and
// AppDef_MyLineTool): LeastSquare, ResolConstraint, Function, BSpFunction, Gradient,
// BSpGradient (tools/tkgeombase_corpus_appparcurves.py).

#include <AppDef_BSpParFunctionOfMyBSplGradientOfBSplineCompute.hxx>
#include <AppDef_MultiLine.hxx>
#include <AppDef_MultiPointConstraint.hxx>
#include <AppDef_MyBSplGradientOfBSplineCompute.hxx>
#include <AppDef_TheFunction.hxx>
#include <AppDef_TheGradient.hxx>
#include <AppDef_TheLeastSquares.hxx>
#include <AppDef_TheResol.hxx>
typedef occ::handle<NCollection_HArray1<AppParCurves_ConstraintCouple>> GbApcConstraints;
// `nb3d nb2d n` then per multipoint `tangent curvature points... [tangents...] [curvatures...]`.
AppDef_MultiLine gb_apc_read_line(GbReader& r){
 const int nb3d=r.integer(),nb2d=r.integer(),n=r.integer();AppDef_MultiLine line(n);
 for(int i=1;i<=n;++i){
  const bool tangent=r.flag(),curvature=r.flag();
  NCollection_Array1<gp_Pnt> p(1,std::max(nb3d,1));NCollection_Array1<gp_Pnt2d> q(1,std::max(nb2d,1));
  for(int k=1;k<=nb3d;++k)p(k)=r.pnt();
  for(int k=1;k<=nb2d;++k)q(k)=r.pnt2d();
  NCollection_Array1<gp_Pnt> p3(1,nb3d);NCollection_Array1<gp_Pnt2d> p2(1,nb2d);
  for(int k=1;k<=nb3d;++k)p3(k)=p(k);
  for(int k=1;k<=nb2d;++k)p2(k)=q(k);
  AppDef_MultiPointConstraint m=nb2d==0?AppDef_MultiPointConstraint(p3):nb3d==0?AppDef_MultiPointConstraint(p2):AppDef_MultiPointConstraint(p3,p2);
  for(int pass=0;pass<2;++pass){
   if(!(pass==0?tangent:curvature))continue;
   for(int k=1;k<=nb3d;++k){gp_Vec v=r.vec();if(pass==0)m.SetTang(k,v);else m.SetCurv(k,v);}
   for(int k=1;k<=nb2d;++k){gp_Vec2d v=r.vec2d();if(pass==0)m.SetTang2d(nb3d+k,v);else m.SetCurv2d(nb3d+k,v);}
  }
  line.SetValue(i,m);
 }
 return line;
}
GbApcConstraints gb_apc_read_constraints(GbReader& r){
 const int n=r.integer();GbApcConstraints c=new NCollection_HArray1<AppParCurves_ConstraintCouple>(1,n);
 for(int i=1;i<=n;++i){const int index=r.integer();c->SetValue(i,AppParCurves_ConstraintCouple(index,AppParCurves_Constraint(r.integer())));}
 return c;
}
NCollection_Array1<double> gb_apc_read_knots(GbReader& r){
 const int n=r.integer();NCollection_Array1<double> k(1,n);for(double& v:k)v=r.real();return k;
}
NCollection_Array1<int> gb_apc_read_mults(GbReader& r){
 const int n=r.integer();NCollection_Array1<int> m(1,n);for(int& v:m)v=r.integer();return m;
}
#define GB_APC_TRY(...) try{__VA_ARGS__;}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
// The results of a least-squares fit, each guarded.
void gb_apc_ls_results(AppDef_TheLeastSquares& ls,bool bspline,int lambdas){
 gb_flag(ls.IsDone());
 GB_APC_TRY(gb_fem_matrix(ls.FunctionMatrix()));
 GB_APC_TRY(gb_fem_matrix(ls.DerivativeFunctionMatrix()));
 gb_apc_ivector(ls.KIndex());gb_fem_matrix(ls.Points());
 if(!ls.IsDone())return;
 gb_fem_matrix(ls.Poles());
 GB_APC_TRY(double f,e3,e2;ls.Error(f,e3,e2);gb_real(f);gb_real(e3);gb_real(e2));
 gb_fem_matrix(ls.Distance());
 GB_APC_TRY(math_Vector g(ls.Points().LowerRow(),ls.Points().UpperRow(),-7.);double f,e3,e2;ls.ErrorGradient(g,f,e3,e2);gb_fem_vector(g);gb_real(f);gb_real(e3);gb_real(e2));
 gb_fem_matrix(ls.Distance());
 if(lambdas&1)gb_real(ls.FirstLambda());
 if(lambdas&2)gb_real(ls.LastLambda());
 GB_APC_TRY(gb_apc_curve(ls.BezierValue()));
 GB_APC_TRY(const AppParCurves_MultiBSpCurve& c=ls.BSplineValue();gb_apc_curve(c);if(bspline)gb_int(c.Degree()));
}
// `apc_ls line fp lp fc lc nbpol mode [knots mults] params variant [vectors, l1, l2] lambdas`.
void gb_apc_ls_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);
 const int fp=r.integer(),lp=r.integer();const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 const int nbpol=r.integer(),mode=r.integer();
 NCollection_Array1<double> knots=mode>=2?gb_apc_read_knots(r):NCollection_Array1<double>(1,1);
 NCollection_Array1<int> mults=mode>=2?gb_apc_read_mults(r):NCollection_Array1<int>(1,1);
 math_Vector u=gb_fem_read_vector(r);const int variant=r.integer();
 std::vector<math_Vector> v;
 for(int i=0;i<(variant==2?2:variant==3?4:0);++i)v.push_back(gb_fem_read_vector(r));
 const double l1=variant>0?r.real():0.,l2=variant>0?r.real():0.;const int lambdas=r.integer();
 try{
  if(mode==0||mode==2){
   AppDef_TheLeastSquares ls=mode==0?AppDef_TheLeastSquares(line,fp,lp,fc,lc,u,nbpol):AppDef_TheLeastSquares(line,knots,mults,fp,lp,fc,lc,u,nbpol);
   gb_apc_ls_results(ls,mode==2,lambdas);
  }else{
   AppDef_TheLeastSquares ls=mode==1?AppDef_TheLeastSquares(line,fp,lp,fc,lc,nbpol):AppDef_TheLeastSquares(line,knots,mults,fp,lp,fc,lc,nbpol);
   if(variant==0)ls.Perform(u);else if(variant==1)ls.Perform(u,l1,l2);
   else if(variant==2)ls.Perform(u,v[0],v[1],l1,l2);else ls.Perform(u,v[0],v[1],v[2],v[3],l1,l2);
   gb_apc_ls_results(ls,mode==3,lambdas);
  }
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `apc_resol line fp lp fc lc constraints nbpol params tolerance`: a Bezier fit, then
// the Uzawa correction of its curve and the constraint derivative.
void gb_apc_resol_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);
 const int fp=r.integer(),lp=r.integer();const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 GbApcConstraints cons=gb_apc_read_constraints(r);const int nbpol=r.integer();math_Vector u=gb_fem_read_vector(r);const double tol=r.real();
 try{
  AppDef_TheLeastSquares ls(line,fp,lp,fc,lc,u,nbpol);
  AppParCurves_MultiCurve curve=ls.BezierValue();math_Matrix a=ls.FunctionMatrix(),da=ls.DerivativeFunctionMatrix();
  AppDef_TheResol resol(line,curve,fp,lp,cons,a,da,tol);
  gb_flag(resol.IsDone());gb_fem_matrix(resol.ConstraintMatrix());
  if(resol.IsDone()){gb_fem_matrix(resol.InverseMatrix());gb_fem_vector(resol.Duale());}
  gb_apc_curve(curve);gb_apc_curve(ls.BezierValue());
  GB_APC_TRY(gb_fem_matrix(resol.ConstraintDerivative(line,u,nbpol-1,da));gb_fem_matrix(resol.ConstraintMatrix()));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
void gb_apc_bsp_extra(int,AppDef_TheFunction&){}
void gb_apc_bsp_extra(int code,AppDef_BSpParFunctionOfMyBSplGradientOfBSplineCompute& f){
 if(code==8)gb_fem_matrix(f.FunctionMatrix());
 else if(code==9)gb_fem_matrix(f.DerivativeFunctionMatrix());
 else gb_apc_ivector(f.Index());
}
// Function calls: 1 Value, 2 Gradient, 3 Values, 4 CurveValue, 5 Error(all),
// 6 MaxError3d/2d, 7 NewParameters, then (BSpFunction) 8 FunctionMatrix, 9
// DerivativeFunctionMatrix, 10 Index.
template<class F,class C> void gb_apc_function_call(GbReader& r,F& f,const C& curve_of,int first,int last,int curves){
 const int code=r.integer();
 if(code<=3){math_Vector x=gb_fem_read_vector(r);double value=-7.;math_Vector g(first,last,-7.);
  if(code==1){gb_flag(f.Value(x,value));gb_real(value);}
  else if(code==2){gb_flag(f.Gradient(x,g));gb_fem_vector(g);}
  else{gb_flag(f.Values(x,value,g));gb_real(value);gb_fem_vector(g);}}
 else if(code==4)curve_of(f);
 else if(code==5){for(int i=first;i<=last;++i)for(int k=1;k<=curves;++k)gb_real(f.Error(i,k));}
 else if(code==6){gb_real(f.MaxError3d());gb_real(f.MaxError2d());}
 else if(code==7)gb_fem_vector(f.NewParameters());
 else gb_apc_bsp_extra(code,f);
}
template<class F,class C> void gb_apc_function_calls(GbReader& r,F& f,const C& curve_of,int first,int last,int curves){
 const int count=r.integer();
 for(int k=0;k<count;++k){GB_APC_TRY(gb_apc_function_call(r,f,curve_of,first,last,curves));}
}
// `apc_function line fp lp constraints params deg calls...`.
void gb_apc_function_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int fp=r.integer(),lp=r.integer();
 GbApcConstraints cons=gb_apc_read_constraints(r);math_Vector u=gb_fem_read_vector(r);const int deg=r.integer();
 const int curves=line.Value(1).NbPoints()+line.Value(1).NbPoints2d();
 try{
  AppDef_TheFunction f(line,fp,lp,cons,u,deg);gb_int(f.NbVariables());
  gb_apc_function_calls(r,f,[](AppDef_TheFunction& g){gb_apc_curve(g.CurveValue());},fp,lp,curves);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `apc_bspfunction line fp lp constraints params knots mults nbpol l1 l2 calls...`.
void gb_apc_bspfunction_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int fp=r.integer(),lp=r.integer();
 GbApcConstraints cons=gb_apc_read_constraints(r);math_Vector u=gb_fem_read_vector(r);
 NCollection_Array1<double> knots=gb_apc_read_knots(r);NCollection_Array1<int> mults=gb_apc_read_mults(r);
 const int nbpol=r.integer();const double l1=r.real(),l2=r.real();
 const int curves=line.Value(1).NbPoints()+line.Value(1).NbPoints2d();
 try{
  AppDef_BSpParFunctionOfMyBSplGradientOfBSplineCompute f(line,fp,lp,cons,u,knots,mults,nbpol);gb_int(f.NbVariables());
  f.SetFirstLambda(l1);f.SetLastLambda(l2);
  gb_apc_function_calls(r,f,[](AppDef_BSpParFunctionOfMyBSplGradientOfBSplineCompute& g){AppParCurves_MultiBSpCurve c=g.CurveValue();gb_apc_curve(c);gb_int(c.Degree());},fp,lp,curves);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
template<class G> void gb_apc_gradient_results(const G& g,const math_Vector& u,int fp,int lp){
 gb_flag(g.IsDone());gb_apc_curve(g.Value());
 for(int j=fp;j<=lp;++j)gb_real(g.Error(j));
 gb_real(g.AverageError());gb_real(g.MaxError3d());gb_real(g.MaxError2d());gb_fem_vector(u);
}
// `apc_gradient line fp lp constraints params deg tol3d tol2d iterations`.
void gb_apc_gradient_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int fp=r.integer(),lp=r.integer();
 GbApcConstraints cons=gb_apc_read_constraints(r);math_Vector u=gb_fem_read_vector(r);
 const int deg=r.integer();const double t3=r.real(),t2=r.real();const int iterations=r.integer();
 try{AppDef_TheGradient g(line,fp,lp,cons,u,deg,t3,t2,iterations);gb_apc_gradient_results(g,u,fp,lp);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `apc_bspgradient line fp lp constraints params knots mults deg tol3d tol2d iterations lambdas l1 l2`.
void gb_apc_bspgradient_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_apc_read_line(r);const int fp=r.integer(),lp=r.integer();
 GbApcConstraints cons=gb_apc_read_constraints(r);math_Vector u=gb_fem_read_vector(r);
 NCollection_Array1<double> knots=gb_apc_read_knots(r);NCollection_Array1<int> mults=gb_apc_read_mults(r);
 const int deg=r.integer();const double t3=r.real(),t2=r.real();const int iterations=r.integer(),with=r.integer();const double l1=r.real(),l2=r.real();
 try{
  if(with){AppDef_MyBSplGradientOfBSplineCompute g(line,fp,lp,cons,u,knots,mults,deg,t3,t2,iterations,l1,l2);gb_apc_gradient_results(g,u,fp,lp);gb_int(g.Value().Degree());}
  else{AppDef_MyBSplGradientOfBSplineCompute g(line,fp,lp,cons,u,knots,mults,deg,t3,t2,iterations);gb_apc_gradient_results(g,u,fp,lp);gb_int(g.Value().Degree());}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
