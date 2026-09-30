// BRepGProp probe operations on the tools of the integrators: the face tool, the domain,
// the Gauss integrator and the Gauss-Kronrod integrand functions, over shapes read by
// tp_shape (tktopalgo_probe_shape.cxx). The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_gprop_tools.rs; tp_gprops is in
// tktopalgo_probe_brepgprop.cxx and tp_index_of in tktopalgo_probe_brepclass3d.cxx.
#include <BRepGProp_Gauss.hxx>
#include <BRepGProp_TFunction.hxx>
#include <BRepGProp_UFunction.hxx>
#include <Precision.hxx>
void tp_knots_out(const occ::handle<NCollection_HArray1<double>>& k){ga_int(k->Lower());ga_int(k->Length());for(double x:k->Array1())ga_real(x);}
// The orders, subdivisions and knots of the loaded face.
void tp_face_orders(BRepGProp_Face& f,double eps){
 GA_TRY(ga_int(f.UIntegrationOrder()))GA_TRY(ga_int(f.VIntegrationOrder()))GA_TRY(ga_int(f.SIntOrder(eps)))
 GA_TRY(int n=f.SUIntSubs();ga_int(n);NCollection_Array1<double> k(1,n+1);f.UKnots(k);for(double x:k)ga_real(x))
 GA_TRY(int n=f.SVIntSubs();ga_int(n);NCollection_Array1<double> k(1,n+1);f.VKnots(k);for(double x:k)ga_real(x))}
// The boundary members after Load(E) (or Load(IsFirst, Iso)).
void tp_face_boundary(BRepGProp_Face& f,double eps){
 double a=f.FirstParameter(),b=f.LastParameter();ga_real(a);ga_real(b);
 GA_TRY(ga_int(f.IntegrationOrder()))GA_TRY(ga_int(f.LIntOrder(eps)))
 GA_TRY(int n=f.LIntSubs();ga_int(n);NCollection_Array1<double> k(1,n+1);f.LKnots(k);for(double x:k)ga_real(x))
 GA_TRY(ga_pnt2d(f.Value2d(0.5*(a+b)));gp_Pnt2d p;gp_Vec2d v;f.D12d(0.37*a+0.63*b,p,v);ga_pnt2d(p);ga_xy(v.XY()))
 GA_TRY(tp_knots_out(f.GetTKnots(a,b)))}
// `shape face span eps`: the face tool: natural restriction, bounds, orders and knots,
// normals at three points of finite bounds, U knots of the bounds, then each domain
// edge (index, orientation, Load, boundary members) and the four iso boundaries.
void tp_bgp_face_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());bool span=r.flag();double eps=r.real();
  BRepGProp_Face f(face,span);ga_int(tp_index_of(s.faces,f.GetFace()));ga_int(f.GetFace().Orientation());
  GA_TRY(ga_flag(f.NaturalRestriction()))double u1,u2,v1,v2;f.Bounds(u1,u2,v1,v2);
  ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2);tp_face_orders(f,eps);
  if(!Precision::IsInfinite(u1)&&!Precision::IsInfinite(u2)&&!Precision::IsInfinite(v1)&&!Precision::IsInfinite(v2)){
   const double at[3][2]={{0.25,0.25},{0.5,0.75},{0.9,0.1}};
   for(auto& w:at){gp_Pnt p;gp_Vec n;GA_TRY(f.Normal(u1+w[0]*(u2-u1),v1+w[1]*(v2-v1),p,n);ga_pnt(p);ga_xyz(n.XYZ()))}
   GA_TRY(tp_knots_out(f.GetUKnots(u1,u2)))GA_TRY(tp_knots_out(f.GetUKnots(u1+0.3*(u2-u1),u2-0.1*(u2-u1))))}
  for(BRepGProp_Domain d(face);d.More();d.Next()){const TopoDS_Edge& e=d.Value();ga_int(tp_index_of(s.edges,e));ga_int(e.Orientation());
   GA_TRY(bool ok=f.Load(e);ga_flag(ok);if(ok)tp_face_boundary(f,eps))}
  for(int iso=0;iso<3;++iso)for(int first=0;first<2;++first){
   GA_TRY(f.Load(first==1,GeomAbs_IsoType(iso));tp_face_boundary(f,eps))})
 ga_end();}
// `shape face span edge umin umax tmin tmax`: GetUKnots and GetTKnots (and their deprecated
// overloads) on arbitrary ranges, the boundary the edge `edge` (-1: the right side).
void tp_bgp_knots_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());bool span=r.flag();int e=r.integer();
  double umin=r.real(),umax=r.real(),tmin=r.real(),tmax=r.real();BRepGProp_Face f(face,span);
  if(e<0)f.Load(false,GeomAbs_IsoU);else ga_flag(f.Load(s.edges.at(e)));
  GA_TRY(tp_knots_out(f.GetUKnots(umin,umax)))GA_TRY(tp_knots_out(f.GetTKnots(tmin,tmax)))
  occ::handle<NCollection_HArray1<double>> k;GA_TRY(tp_u_knots_into(f,umin,umax,k);tp_knots_out(k))
  GA_TRY(tp_t_knots_into(f,tmin,tmax,k);tp_knots_out(k)))
 ga_end();}
// `shape face edge which lower n`: UKnots (0), VKnots (1) or LKnots (2, after Load of the
// edge) into an array of n values indexed from `lower`, prefilled with -1.
void tp_bgp_arrays_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());int e=r.integer(),which=r.integer();
  int lower=r.integer(),n=r.integer();BRepGProp_Face f(face);if(e>=0)f.Load(s.edges.at(e));
  NCollection_Array1<double> k(lower,lower+n-1);k.Init(-1.);
  GA_TRY(if(which==0)f.UKnots(k);else if(which==1)f.VKnots(k);else f.LKnots(k))for(double x:k)ga_real(x))
 ga_end();}
// `shape face`: the domain: the edges it returns, Value past the end, Init and the first
// edge again, then an empty domain (More, Value).
void tp_bgp_domain_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());BRepGProp_Domain d(face);
  for(;d.More();d.Next()){ga_int(tp_index_of(s.edges,d.Value()));ga_int(d.Value().Orientation());}
  GA_TRY(ga_int(tp_index_of(s.edges,d.Value())))d.Init();ga_flag(d.More());if(d.More())ga_int(tp_index_of(s.edges,d.Value()));
  BRepGProp_Domain empty;ga_flag(empty.More());GA_TRY(ga_flag(empty.Value().IsNull()))
  BRepGProp_Domain later;later.Init(face);ga_flag(later.More()))
 ga_end();}
// `shape face type overload x y z eps bypoint n coeffs...`: one BRepGProp_Gauss Compute
// (0 natural with coefficients, 1 natural surface, 2 domain with coefficients, 3 domain
// surface, 4 adaptive with coefficients, 5 adaptive surface) on outputs prefilled with
// mass -7, centre (1, 2, 3) and 5 times the identity; the error of the adaptive ones.
void tp_bgp_gauss_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());
  BRepGProp_Gauss g(BRepGProp_Gauss::GaussType(r.integer()));int overload=r.integer();gp_Pnt loc=r.pnt();double eps=r.real();
  bool byPoint=r.flag();std::vector<double> coeff=r.reals();BRepGProp_Face F(face);BRepGProp_Domain D(face);
  double mass=-7.;gp_Pnt c(1,2,3);gp_Mat m(gp_XYZ(5,0,0),gp_XYZ(0,5,0),gp_XYZ(0,0,5));
  GA_TRY(if(overload==0)g.Compute(F,loc,coeff.data(),byPoint,mass,c,m);else if(overload==1)g.Compute(F,loc,mass,c,m);
   else if(overload==2)g.Compute(F,D,loc,coeff.data(),byPoint,mass,c,m);else if(overload==3)g.Compute(F,D,loc,mass,c,m);
   else if(overload==4)ga_real(g.Compute(F,D,loc,eps,coeff.data(),byPoint,mass,c,m));else ga_real(g.Compute(F,D,loc,eps,mass,c,m)))
  ga_real(mass);ga_pnt(c);for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)ga_real(m(i,j)))
 ga_end();}
// `shape face edge span bypoint x y z n coeffs... type umin tol nu (x v)... nt (t)...`: the
// U function (after SetValueType and SetVParam) and the T function (after
// SetValueType, SetNbKronrodPoints and Init) of the face with the boundary of `edge`
// (-1: the right side): each value's flag and value, then ErrorReached and
// AbsolutError.
void tp_bgp_functions_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);const TopoDS_Face& face=s.faces.at(r.integer());int e=r.integer();bool span=r.flag();
  bool byPoint=r.flag();gp_Pnt vertex=r.pnt();std::vector<double> coeff=r.reals();GProp_ValueType type=GProp_ValueType(r.integer());
  double umin=r.real(),tol=r.real();BRepGProp_Face f(face,span);if(e<0)f.Load(false,GeomAbs_IsoU);else ga_flag(f.Load(s.edges.at(e)));
  BRepGProp_UFunction u(f,vertex,byPoint,coeff.data());u.SetValueType(type);int nu=r.integer();
  for(int i=0;i<nu;++i){double x=r.real();u.SetVParam(r.real());double v=0.;GA_TRY(ga_flag(u.Value(x,v));ga_real(v))}
  BRepGProp_TFunction t(f,vertex,byPoint,coeff.data(),umin,tol);t.SetValueType(type);t.SetNbKronrodPoints(4);t.Init();
  t.SetTolerance(tol);int nt=r.integer();
  for(int i=0;i<nt;++i){double x=r.real(),v=0.;GA_TRY(ga_flag(t.Value(x,v));ga_real(v))}
  ga_real(t.ErrorReached());ga_real(t.AbsolutError());ga_int(t.GetStateNumber()))
 ga_end();}
