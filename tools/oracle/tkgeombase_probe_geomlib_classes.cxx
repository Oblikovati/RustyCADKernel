// TKGeomBase oracle: the classes of GeomLib (tkgeombase_probe.cxx), with the GeomTools
// builders and the printers of tkgeombase_probe_geomconvert.cxx. The Rust twin is
// crates/tkgeombase/examples/gb_probe/geom_lib/classes.rs.
#include <AdvApprox_ApproxAFunction.hxx>
#include <AdvApprox_EvaluatorFunction.hxx>
#include <GeomLib_Check2dBSplineCurve.hxx>
#include <GeomLib_CheckBSplineCurve.hxx>
#include <GeomLib_CheckCurveOnSurface.hxx>
#include <GeomLib_DenominatorMultiplier.hxx>
#include <GeomLib_Interpolate.hxx>
#include <GeomLib_IsPlanarSurface.hxx>
#include <GeomLib_LogSample.hxx>
#include <GeomLib_MakeCurvefromApprox.hxx>
#include <GeomLib_PolyFunc.hxx>
#include <GeomLib_Tool.hxx>
// gl_check_bspline Dim Tol AngTol FixFirst FixLast Curve: GeomLib_CheckBSplineCurve
// (Dim 3) or GeomLib_Check2dBSplineCurve (2); IsDone, NeedTangentFix, FixedTangent,
// FixTangent, then the input curve, which FixTangent modifies.
void gb_gl_check_bspline_case(std::istream& in){
 GbReader r{in};const int dim=r.integer();const double tol=r.real(),ang=r.real();const bool ff=r.flag(),fl=r.flag();
 if(dim==3){GB_TRY(occ::handle<Geom_BSplineCurve> c=gb_gcv_read_bspline(r);GeomLib_CheckBSplineCurve k(c,tol,ang);bool a=false,b=false;
  gb_flag(k.IsDone());k.NeedTangentFix(a,b);gb_flag(a);gb_flag(b);GB_TRY(gb_gcv_bspline(k.FixedTangent(ff,fl)));
  GB_TRY(k.FixTangent(ff,fl));gb_gcv_bspline(c));gb_end();return;}
 GB_TRY(occ::handle<Geom2d_BSplineCurve> c=occ::down_cast<Geom2d_BSplineCurve>(gt_curve2d(r));GeomLib_Check2dBSplineCurve k(c,tol,ang);bool a=false,b=false;
  gb_flag(k.IsDone());k.NeedTangentFix(a,b);gb_flag(a);gb_flag(b);GB_TRY(gb_g2c_bspline(k.FixedTangent(ff,fl)));
  GB_TRY(k.FixTangent(ff,fl));gb_g2c_bspline(c));gb_end();
}
// gl_cos Mode First Last First2d Last2d TolRange Parallel Curve Curve2d Surface: the
// deviation of the 3D curve on [First, Last] from the pcurve on [First2d, Last2d] on
// the surface; mode 0 constructs with the curve, 1 uses Init(curve), 2 calls Init()
// before Perform. IsParallel, IsDone, ErrorStatus, MaxDistance, MaxParameter.
void gb_gl_cos_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double f=r.real(),l=r.real(),f2=r.real(),l2=r.real(),tol=r.real();const bool par=r.flag();
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);occ::handle<Geom2d_Curve> p=gt_curve2d(r);occ::handle<Geom_Surface> s=gt_surface(r);
  occ::handle<Adaptor3d_Curve> ca=new GeomAdaptor_Curve(c,f,l);
  occ::handle<Adaptor3d_CurveOnSurface> cos=new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(p,f2,l2),new GeomAdaptor_Surface(s));
  std::unique_ptr<GeomLib_CheckCurveOnSurface> k(mode==0?new GeomLib_CheckCurveOnSurface(ca,tol):new GeomLib_CheckCurveOnSurface());
  if(mode!=0)k->Init(ca,tol);if(mode==2)k->Init();k->SetParallel(par);gb_flag(k->IsParallel());k->Perform(cos);
  gb_flag(k->IsDone());gb_int(k->ErrorStatus());gb_real(k->MaxDistance());gb_real(k->MaxParameter()));
 gb_end();
}
// gl_denom NKnots Knot... NUV (U V)... Surface: DenominatorMultiplier values.
void gb_gl_denom_case(std::istream& in){
 GbReader r{in};const int nk=r.integer();NCollection_Array1<double> k(1,nk);for(int i=1;i<=nk;++i)k(i)=r.real();
 const int n=r.integer();std::vector<double> uv(2*n);for(double& x:uv)x=r.real();
 GB_TRY(occ::handle<Geom_BSplineSurface> s=gb_gcv_read_bspline_surface(r);GeomLib_DenominatorMultiplier d(s,k);
  for(int i=0;i<n;++i){GB_TRY(gb_real(d.Value(uv[2*i],uv[2*i+1])));});
 gb_end();
}
// gl_interpolate Degree N Point... Param...: GeomLib_Interpolate; IsDone, Error, Curve.
void gb_gl_interpolate_case(std::istream& in){
 GbReader r{in};const int deg=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> t(1,n);
 for(int i=1;i<=n;++i)p(i)=r.pnt();for(int i=1;i<=n;++i)t(i)=r.real();
 GB_TRY(GeomLib_Interpolate g(deg,n,p,t);gb_flag(g.IsDone());if(g.IsDone())gb_gcv_bspline(g.Curve());else gb_int(g.Error()));gb_end();
}
// gl_is_planar Tol Surface: IsPlanar and Plan.
void gb_gl_is_planar_case(std::istream& in){
 GbReader r{in};const double tol=r.real();
 GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);GeomLib_IsPlanarSurface k(s,tol);gb_flag(k.IsPlanar());GB_TRY(gb_pln(k.Plan())));gb_end();
}
// gl_log_sample A B N: GetParameter(0 .. N + 1).
void gb_gl_log_sample_case(std::istream& in){
 GbReader r{in};const double a=r.real(),b=r.real();const int n=r.integer();GeomLib_LogSample s(a,b,n);
 for(int i=0;i<=n+1;++i){GB_TRY(gb_real(s.GetParameter(i)));}gb_end();
}
// gl_poly_func Lower N Coeff... X: Value, Derivative and Values.
void gb_gl_poly_func_case(std::istream& in){
 GbReader r{in};const int lower=r.integer(),n=r.integer();math_Vector c(lower,lower+n-1);for(int i=lower;i<lower+n;++i)c(i)=r.real();const double x=r.real();
 GB_TRY(GeomLib_PolyFunc f(c);double v=GB_GCV_UNSET,d=GB_GCV_UNSET;gb_flag(f.Value(x,v));gb_real(v);gb_flag(f.Derivative(x,d));gb_real(d);
  v=d=GB_GCV_UNSET;gb_flag(f.Values(x,v,d));gb_real(v);gb_real(d));gb_end();
}
// gl_tool Mode MaxDist Point Geometry: Parameter of a curve (mode 0), Parameters of a
// surface (1) or Parameter of a 2D curve (2, the point's Z ignored).
void gb_gl_tool_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double md=r.real();const gp_Pnt p=r.pnt();double u=GB_GCV_UNSET,v=GB_GCV_UNSET;
 GB_TRY(if(mode==0){occ::handle<Geom_Curve> c=gt_curve(r);gb_flag(GeomLib_Tool::Parameter(c,p,md,u));}
  else if(mode==1){occ::handle<Geom_Surface> s=gt_surface(r);gb_flag(GeomLib_Tool::Parameters(s,p,md,u,v));}
  else{occ::handle<Geom2d_Curve> c=gt_curve2d(r);gb_flag(GeomLib_Tool::Parameter(c,gp_Pnt2d(p.X(),p.Y()),md,u));}
  gb_real(u);gb_real(v));gb_end();
}
// gl_deviation Mode First Last Start NbIters NbSub Curve2d: the Newton ComputeDeviation
// (mode 0) with every output, or the PSO one (1) with the parameter.
void gb_gl_deviation_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double f=r.real(),l=r.real(),st=r.real();const int it=r.integer(),sub=r.integer();
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);Geom2dAdaptor_Curve a(c);double prm=GB_GCV_UNSET;
  if(mode==0){gp_Pnt2d pt(GB_GCV_UNSET,GB_GCV_UNSET);gp_Vec2d v(GB_GCV_UNSET,GB_GCV_UNSET);gp_Lin2d ln;
   gb_real(GeomLib_Tool::ComputeDeviation(a,f,l,st,it,&prm,&pt,&v,&ln));gb_real(prm);gb_pnt2d(pt);gb_vec2d(v);gb_lin2d(ln);}
  else{gb_real(GeomLib_Tool::ComputeDeviation(a,f,l,sub,it,&prm));gb_real(prm);});
 gb_end();
}
// Coordinate c of the probe function `kind` at t, derivative order d (0 .. 2): 0 a
// cubic, 1 sine/cosine waves, otherwise a helix; the Rust twin defines it identically.
double gb_gl_aa_value(int kind,int c,double t,int d){
 const double s=c+1.0;
 if(kind==0){if(d==0)return ((s*t-2.0)*t+0.5)*t+c;if(d==1)return (3.0*s*t-4.0)*t+0.5;return 6.0*s*t-4.0;}
 if(kind==1){if(d==0)return std::sin(s*t)+c*std::cos(0.5*t);if(d==1)return s*std::cos(s*t)-0.5*c*std::sin(0.5*t);
  return -s*s*std::sin(s*t)-0.25*c*std::cos(0.5*t);}
 const double rad=2.0+(c/3);const int m=c%3;
 if(m==0){if(d==0)return rad*std::cos(t);if(d==1)return -rad*std::sin(t);return -rad*std::cos(t);}
 if(m==1){if(d==0)return rad*std::sin(t);if(d==1)return rad*std::cos(t);return -rad*std::sin(t);}
 if(d==0)return 1.0+0.3*t;if(d==1)return 0.3;return 0.0;
}
class GbGlEvaluator : public AdvApprox_EvaluatorFunction{
public:
 int kind;
 explicit GbGlEvaluator(int k):kind(k){}
 void Evaluate(int* Dimension,double[2],double* Parameter,int* DerivativeRequest,double* Result,int* ErrorCode) override{
  *ErrorCode=0;for(int c=0;c<*Dimension;++c)Result[c]=gb_gl_aa_value(kind,c,*Parameter,*DerivativeRequest);
 }
};
// gl_make_curve Kind N1 N2 N3 Tol First Last Continuity MaxDeg MaxSeg: the AdvApprox
// approximation of the probe function, then every GeomLib_MakeCurvefromApprox member
// over the indices 0 .. N + 1 of each space kind present (the weighted forms with the
// first 1D space); a member of an absent kind dereferences a null table natively.
void gb_gl_make_curve_case(std::istream& in){
 GbReader r{in};GbGlEvaluator f(r.integer());const int n1=r.integer(),n2=r.integer(),n3=r.integer();const double tol=r.real(),first=r.real(),last=r.real();
 const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());const int deg=r.integer(),seg=r.integer();
 auto tols=[&](int n){occ::handle<NCollection_HArray1<double>> h;if(n>0){h=new NCollection_HArray1<double>(1,n);h->Init(tol);}return h;};
 GB_TRY(AdvApprox_ApproxAFunction a(n1,n2,n3,tols(n1),tols(n2),tols(n3),first,last,cont,deg,seg,f);GeomLib_MakeCurvefromApprox m(a);
  gb_flag(m.IsDone());gb_int(m.Nb1DSpaces());gb_int(m.Nb2DSpaces());gb_int(m.Nb3DSpaces());
  if(n2>0)for(int i=0;i<=n2+1;++i){GB_TRY(gb_g2c_bspline(m.Curve2d(i)));if(n1>0){GB_TRY(gb_g2c_bspline(m.Curve2d(1,i)));}}
  if(n1>0)for(int i=0;i<=n1+1;++i){GB_TRY(gb_g2c_bspline(m.Curve2dFromTwo1d(i,2)));}
  if(n3>0)for(int i=0;i<=n3+1;++i){GB_TRY(gb_gcv_bspline(m.Curve(i)));if(n1>0){GB_TRY(gb_gcv_bspline(m.Curve(1,i)));}});
 gb_end();
}
