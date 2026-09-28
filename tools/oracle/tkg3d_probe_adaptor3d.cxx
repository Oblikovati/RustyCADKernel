// TKG3d oracle: Adaptor3d_CurveOnSurface, Adaptor3d_IsoCurve, Adaptor3d_InterFunc and
// Adaptor3d_HSurfaceTool (tkg3d_probe.cxx).
// A 2D curve specification, read recursively:
// 0 line P D | 1 circle P X sense R | 2 ellipse P X sense A B | 3 hyperbola P X sense A B |
// 4 parabola P X sense F | 5 bezier rational n (x y w)*n |
// 6 bspline rational periodic degree nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 <curve>,
// where P X are gp_Ax2d location and direction.
// A 2D adaptor: 0 Geom2dAdaptor_Curve(C) <curve> | 1 Geom2dAdaptor_Curve(C, u1, u2) u1 u2 <curve> |
// 2 Adaptor2d_Line2d(P, D, u1, u2).
// A surface adaptor: 0 GeomAdaptor_Surface(S) <surface> |
// 1 GeomAdaptor_Surface(S, u1, u2, v1, v2, tu, tv) <surface> | 2 Adaptor3d_Surface (the base class) |
// 3 GeomAdaptor_SurfaceOfLinearExtrusion(GeomAdaptor_Curve(C), D) D <curve>.
gp_Pnt2d a3_pnt2(G3Reader& r){double x=r.real();return gp_Pnt2d(x,r.real());}
gp_Dir2d a3_dir2(G3Reader& r){double x=r.real();return gp_Dir2d(x,r.real());}
occ::handle<Geom2d_Curve> a3_curve2d(G3Reader& r){
 int k=r.integer();
 if(k==0){gp_Pnt2d p=a3_pnt2(r);return new Geom2d_Line(p,a3_dir2(r));}
 if(k>=1&&k<=4){gp_Pnt2d p=a3_pnt2(r);gp_Ax2d a(p,a3_dir2(r));bool s=r.flag();double x=r.real();
  if(k==1)return new Geom2d_Circle(a,x,s);
  if(k==4)return new Geom2d_Parabola(a,x,s);
  double y=r.real();if(k==2)return new Geom2d_Ellipse(a,x,y,s);return new Geom2d_Hyperbola(a,x,y,s);}
 if(k==5){bool rat=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);
  for(int i=1;i<=n;++i){p(i)=a3_pnt2(r);w(i)=r.real();}
  if(rat)return new Geom2d_BezierCurve(p,w);return new Geom2d_BezierCurve(p);}
 if(k==6){bool rat=r.flag(),per=r.flag();int d=r.integer(),nk=r.integer();
  NCollection_Array1<double> kn(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){kn(i)=r.real();m(i)=r.integer();}
  int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
  for(int i=1;i<=np;++i){p(i)=a3_pnt2(r);w(i)=r.real();}
  if(rat)return new Geom2d_BSplineCurve(p,w,kn,m,d,per);return new Geom2d_BSplineCurve(p,kn,m,d,per);}
 if(k==7){double u1=r.real(),u2=r.real();occ::handle<Geom2d_Curve> c=a3_curve2d(r);return new Geom2d_TrimmedCurve(c,u1,u2);}
 throw Standard_ProgramError("unknown 2D curve kind");
}
occ::handle<Adaptor2d_Curve2d> a3_adaptor2d(G3Reader& r){
 int k=r.integer();
 if(k==0)return new Geom2dAdaptor_Curve(a3_curve2d(r));
 if(k==1){double u1=r.real(),u2=r.real();return new Geom2dAdaptor_Curve(a3_curve2d(r),u1,u2);}
 if(k==2){gp_Pnt2d p=a3_pnt2(r);gp_Dir2d d=a3_dir2(r);double u1=r.real();return new Adaptor2d_Line2d(p,d,u1,r.real());}
 throw Standard_ProgramError("unknown 2D adaptor kind");
}
occ::handle<Adaptor3d_Surface> a3_surface(G3Reader& r){
 int k=r.integer();
 if(k==0)return new GeomAdaptor_Surface(g3_surface(r));
 if(k==1){double b[6];for(double& v:b)v=r.real();return new GeomAdaptor_Surface(g3_surface(r),b[0],b[1],b[2],b[3],b[4],b[5]);}
 if(k==2)return new Adaptor3d_Surface();
 if(k==3){gp_Dir d=r.dir();occ::handle<Adaptor3d_Curve> c=new GeomAdaptor_Curve(g3_curve(r));
  return new GeomAdaptor_SurfaceOfLinearExtrusion(c,d);}
 throw Standard_ProgramError("unknown surface adaptor kind");
}
// Reads a 2D adaptor and a surface adaptor, reporting a construction failure.
bool a3_inputs(G3Reader& r,occ::handle<Adaptor2d_Curve2d>& c,occ::handle<Adaptor3d_Surface>& s){
 try{c=a3_adaptor2d(r);s=a3_surface(r);return true;}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return false;}
}
void a3_circ(const gp_Circ& c){g3_ax2(c.Position());g3_real(c.Radius());}
void c3_curve_intervals(const occ::handle<Adaptor3d_Curve>& a){
 for(int s=0;s<=6;++s){int n=0;
  try{n=a->NbIntervals(GeomAbs_Shape(s));g3_int(n);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';continue;}
  NCollection_Array1<double> t(1,n+1);G3_TRY(a->Intervals(t,GeomAbs_Shape(s));for(double v:t)g3_real(v));}
}
// Whether a spline query would dereference a null 2D spline: a curve on a plane passes
// the Bezier()/BSpline() handle of its 2D adaptor, null for another 2D kind.
bool a3_null_spline(const occ::handle<Adaptor3d_Curve>& a,GeomAbs_CurveType kind){
 occ::handle<Adaptor3d_CurveOnSurface> c=occ::down_cast<Adaptor3d_CurveOnSurface>(a);
 try{if(c.IsNull()||c->GetSurface()->GetType()!=GeomAbs_Plane)return false;
  return c->GetCurve()->GetType()!=kind&&!c->GetCurve()->IsKind(STANDARD_TYPE(Adaptor2d_Line2d));}
 catch(const Standard_Failure&){return false;}
}
void c3_curve_kinds(const occ::handle<Adaptor3d_Curve>& a){
 G3_TRY(g3_int(a->GetType()));G3_TRY(g3_ax1(a->Line().Position()));G3_TRY(a3_circ(a->Circle()));
 G3_TRY(gp_Elips e=a->Ellipse();g3_ax2(e.Position());g3_real(e.MajorRadius());g3_real(e.MinorRadius()));
 G3_TRY(gp_Hypr h=a->Hyperbola();g3_ax2(h.Position());g3_real(h.MajorRadius());g3_real(h.MinorRadius()));
 G3_TRY(gp_Parab p=a->Parabola();g3_ax2(p.Position());g3_real(p.Focal()));
 G3_TRY(g3_int(a->Degree()));G3_TRY(g3_flag(a->IsRational()));G3_TRY(g3_int(a->NbPoles()));G3_TRY(g3_int(a->NbKnots()));
 if(!a3_null_spline(a,GeomAbs_BezierCurve))G3_TRY(occ::handle<Geom_BezierCurve> b=a->Bezier();g3_name(b);if(!b.IsNull()){g3_flag(b->IsRational());
  for(int i=1;i<=b->NbPoles();++i){g3_pnt(b->Pole(i));g3_real(b->Weight(i));}});
 if(!a3_null_spline(a,GeomAbs_BSplineCurve))G3_TRY(occ::handle<Geom_BSplineCurve> b=a->BSpline();g3_name(b);if(!b.IsNull()){g3_flag(b->IsRational());g3_flag(b->IsPeriodic());
  g3_int(b->Degree());for(int i=1;i<=b->NbKnots();++i){g3_real(b->Knot(i));g3_int(b->Multiplicity(i));}
  for(int i=1;i<=b->NbPoles();++i){g3_pnt(b->Pole(i));g3_real(b->Weight(i));}});
}
void c3_curve_evaluations(const occ::handle<Adaptor3d_Curve>& a,double u){
 G3_TRY(g3_pnt(a->EvalD0(u)));
 G3_TRY(auto e=a->EvalD1(u);g3_pnt(e.Point);g3_vec(e.D1));
 G3_TRY(auto e=a->EvalD2(u);g3_pnt(e.Point);g3_vec(e.D1);g3_vec(e.D2));
 G3_TRY(auto e=a->EvalD3(u);g3_pnt(e.Point);g3_vec(e.D1);g3_vec(e.D2);g3_vec(e.D3));
 for(int n=0;n<=4;++n)G3_TRY(g3_vec(a->EvalDN(u,n)));
 gp_Pnt p;gp_Vec v1,v2,v3;
 G3_TRY(g3_pnt(a->Value(u)));G3_TRY(a->D0(u,p);g3_pnt(p));G3_TRY(a->D1(u,p,v1);g3_pnt(p);g3_vec(v1));
 G3_TRY(a->D2(u,p,v1,v2);g3_pnt(p);g3_vec(v1);g3_vec(v2));G3_TRY(a->D3(u,p,v1,v2,v3);g3_pnt(p);g3_vec(v1);g3_vec(v2);g3_vec(v3));
 G3_TRY(g3_vec(a->DN(u,2)));
}
// The curve-level summary of an Adaptor3d_Curve: bounds, continuity and intervals (twice,
// the second answer coming from a cache for a curve on a surface), closure, periodicity,
// resolutions, kind queries, then the evaluators at the bounds and at every parameter.
void c3_curve_summary(const occ::handle<Adaptor3d_Curve>& a,const std::vector<double>& params){
 std::cout<<a->DynamicType()->Name()<<' ';G3_TRY(g3_real(a->FirstParameter()));G3_TRY(g3_real(a->LastParameter()));
 G3_TRY(g3_int(a->Continuity()));c3_curve_intervals(a);c3_curve_intervals(a);
 G3_TRY(g3_flag(a->IsClosed()));G3_TRY(g3_flag(a->IsPeriodic()));G3_TRY(g3_real(a->Period()));
 for(double r:{1e-7,1e-3,0.5})G3_TRY(g3_real(a->Resolution(r)));c3_curve_kinds(a);
 G3_TRY(c3_curve_evaluations(a,a->FirstParameter()));G3_TRY(c3_curve_evaluations(a,a->LastParameter()));
 for(double u:params)c3_curve_evaluations(a,u);
}
// Intervals into an array of a wrong length (the native assert). An array whose lower
// bound is not 1 is written out of its bounds natively (unchecked raw indices).
void a3_interval_arrays(const occ::handle<Adaptor3d_Curve>& a){
 int n=0;try{n=a->NbIntervals(GeomAbs_C2);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';return;}
 {NCollection_Array1<double> t(1,n+2);G3_TRY(a->Intervals(t,GeomAbs_C2);g3_real(t(1)));}
}
// Adaptor3d_CurveOnSurface on one curve and surface: summary, intervals arrays, Trim and
// ShallowCopy summaries, GetCurve/GetSurface identity.
void a3_cos_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor2d_Curve2d> c;occ::handle<Adaptor3d_Surface> s;if(!a3_inputs(r,c,s))return;
 std::vector<double> u=r.reals();double t1=r.real(),t2=r.real();
 occ::handle<Adaptor3d_CurveOnSurface> a;
 try{a=new Adaptor3d_CurveOnSurface(c,s);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 g3_flag(a->GetCurve()==c);g3_flag(a->GetSurface()==s);c3_curve_summary(a,u);a3_interval_arrays(a);
 G3_TRY(occ::handle<Adaptor3d_Curve> t=a->Trim(t1,t2,1e-7);c3_curve_summary(t,u));
 G3_TRY(occ::handle<Adaptor3d_Curve> h=a->ShallowCopy();c3_curve_summary(h,u));std::cout<<'\n';
}
// Load sequences of Adaptor3d_CurveOnSurface: the empty adaptor, the surface alone, the
// curve on it, a second surface (EvalKPart again, the end patches kept), Load(C, S),
// ChangeCurve/ChangeSurface (nothing re-derived, the interval cache kept).
void a3_cos_load_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor2d_Curve2d> c1,c2;occ::handle<Adaptor3d_Surface> s1,s2;
 if(!a3_inputs(r,c1,s1)||!a3_inputs(r,c2,s2))return;std::vector<double> u=r.reals();
 occ::handle<Adaptor3d_CurveOnSurface> a=new Adaptor3d_CurveOnSurface();g3_flag(a->GetCurve().IsNull());g3_flag(a->GetSurface().IsNull());
 // The other members dereference the null curve natively.
 G3_TRY(g3_int(a->GetType()));
 G3_TRY(a->Load(s1);g3_int(a->GetType()));G3_TRY(a->Load(c1);c3_curve_summary(a,u));
 G3_TRY(a->Load(s2);c3_curve_summary(a,u));G3_TRY(a->Load(c2,s1);c3_curve_summary(a,u));
 G3_TRY(g3_int(a->NbIntervals(GeomAbs_C1)));a->ChangeCurve()=c1;a->ChangeSurface()=s2;g3_flag(a->GetCurve()==c1);
 G3_TRY(c3_curve_summary(a,u));
 G3_TRY(occ::handle<Adaptor3d_CurveOnSurface> b=new Adaptor3d_CurveOnSurface(s2);g3_int(b->GetType());b->Load(c2);c3_curve_summary(b,u));
 std::cout<<'\n';
}
// Adaptor3d_IsoCurve: every constructor, Load(S), Load(Iso, Param) and Load(Iso, Param, W1, W2),
// the accessors, Trim and ShallowCopy.
void a3_iso_summary(const occ::handle<Adaptor3d_IsoCurve>& a,const std::vector<double>& u){
 g3_int(a->Iso());g3_real(a->Parameter());g3_flag(a->Surface().IsNull());c3_curve_summary(a,u);
}
void a3_iso_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor3d_Surface> s;
 try{s=a3_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 int iso=r.integer();double p=r.real(),w1=r.real(),w2=r.real();std::vector<double> u=r.reals();double t1=r.real(),t2=r.real();
 // An empty iso curve: only the members that do not dereference the null surface.
 occ::handle<Adaptor3d_IsoCurve> e=new Adaptor3d_IsoCurve();g3_int(e->Iso());g3_real(e->Parameter());g3_flag(e->Surface().IsNull());
 g3_real(e->FirstParameter());g3_real(e->LastParameter());G3_TRY(g3_int(e->Continuity()));G3_TRY(g3_int(e->NbIntervals(GeomAbs_C1)));
 G3_TRY(g3_flag(e->IsClosed()));G3_TRY(g3_real(e->Resolution(0.5)));G3_TRY(g3_pnt(e->Value(0.)));
 occ::handle<Adaptor3d_IsoCurve> b=new Adaptor3d_IsoCurve(s);a3_iso_summary(b,u);
 G3_TRY(occ::handle<Adaptor3d_IsoCurve> a=new Adaptor3d_IsoCurve(s,GeomAbs_IsoType(iso),p);a3_iso_summary(a,u);
  G3_TRY(c3_curve_summary(a->Trim(t1,t2,1e-7),u));G3_TRY(c3_curve_summary(a->ShallowCopy(),u)));
 G3_TRY(occ::handle<Adaptor3d_IsoCurve> a=new Adaptor3d_IsoCurve(s,GeomAbs_IsoType(iso),p,w1,w2);a3_iso_summary(a,u));
 G3_TRY(b->Load(GeomAbs_IsoType(iso),p,w1,w2);a3_iso_summary(b,u));G3_TRY(b->Load(GeomAbs_IsoType(iso),p);a3_iso_summary(b,u));
 b->Load(s);a3_iso_summary(b,u);std::cout<<'\n';
}
// Adaptor3d_InterFunc on a 2D adaptor: construction (Fix must be 1 or 2), Value,
// Derivative and Values at every parameter.
void a3_interfunc_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor2d_Curve2d> c;
 try{c=a3_adaptor2d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 double v=r.real();int fix=r.integer();std::vector<double> x=r.reals();
 try{Adaptor3d_InterFunc f(c,v,fix);
  for(double t:x){double a=0,b=0;G3_TRY(g3_flag(f.Value(t,a));g3_real(a));G3_TRY(g3_flag(f.Derivative(t,b));g3_real(b));
   a=0;b=0;G3_TRY(g3_flag(f.Values(t,a,b));g3_real(a);g3_real(b));}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 std::cout<<'\n';
}
void a3_surface_d3(const occ::handle<Adaptor3d_Surface>& s,double u,double v){
 gp_Pnt p;gp_Vec d[9];
 Adaptor3d_HSurfaceTool::D3(s,u,v,p,d[0],d[1],d[2],d[3],d[4],d[5],d[6],d[7],d[8]);g3_pnt(p);for(const gp_Vec& x:d)g3_vec(x);
}
// Adaptor3d_HSurfaceTool: every forwarding member, NbSamplesU/V (both overloads) and IsSurfG1.
void a3_hstool_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor3d_Surface> s;
 try{s=a3_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> uv=r.reals();std::vector<double> ranges=r.reals();
 typedef Adaptor3d_HSurfaceTool T;
 G3_TRY(g3_real(T::FirstUParameter(s)));G3_TRY(g3_real(T::LastUParameter(s)));G3_TRY(g3_real(T::FirstVParameter(s)));
 G3_TRY(g3_real(T::LastVParameter(s)));
 for(int k=0;k<=6;++k){int n=0;
  try{n=T::NbUIntervals(s,GeomAbs_Shape(k));g3_int(n);NCollection_Array1<double> t(1,n+1);T::UIntervals(s,t,GeomAbs_Shape(k));for(double x:t)g3_real(x);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try{n=T::NbVIntervals(s,GeomAbs_Shape(k));g3_int(n);NCollection_Array1<double> t(1,n+1);T::VIntervals(s,t,GeomAbs_Shape(k));for(double x:t)g3_real(x);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
 G3_TRY(g3_real(T::UTrim(s,0.,0.5,1e-7)->LastUParameter()));G3_TRY(g3_real(T::VTrim(s,0.,0.5,1e-7)->LastVParameter()));
 G3_TRY(g3_flag(T::IsUClosed(s)));G3_TRY(g3_flag(T::IsVClosed(s)));G3_TRY(g3_flag(T::IsUPeriodic(s)));G3_TRY(g3_real(T::UPeriod(s)));
 G3_TRY(g3_flag(T::IsVPeriodic(s)));G3_TRY(g3_real(T::VPeriod(s)));
 for(size_t i=0;i+1<uv.size();i+=2){double u=uv[i],v=uv[i+1];gp_Pnt p;gp_Vec a,b,c,d,e,f;
  G3_TRY(g3_pnt(T::Value(s,u,v)));G3_TRY(T::D0(s,u,v,p);g3_pnt(p));G3_TRY(T::D1(s,u,v,p,a,b);g3_pnt(p);g3_vec(a);g3_vec(b));
  G3_TRY(T::D2(s,u,v,p,a,b,c,d,e);g3_pnt(p);g3_vec(a);g3_vec(b);g3_vec(c);g3_vec(d);g3_vec(e));G3_TRY(a3_surface_d3(s,u,v));
  G3_TRY(g3_vec(T::DN(s,u,v,1,1)));(void)f;}
 for(double x:{1e-7,0.5})G3_TRY(g3_real(T::UResolution(s,x));g3_real(T::VResolution(s,x)));
 G3_TRY(g3_int(T::GetType(s)));G3_TRY(g3_ax3(T::Plane(s).Position()));G3_TRY(gp_Cylinder c=T::Cylinder(s);g3_ax3(c.Position());g3_real(c.Radius()));
 G3_TRY(gp_Cone c=T::Cone(s);g3_ax3(c.Position());g3_real(c.RefRadius());g3_real(c.SemiAngle()));
 G3_TRY(gp_Torus c=T::Torus(s);g3_ax3(c.Position());g3_real(c.MajorRadius());g3_real(c.MinorRadius()));
 G3_TRY(gp_Sphere c=T::Sphere(s);g3_ax3(c.Position());g3_real(c.Radius()));
 G3_TRY(g3_name(T::Bezier(s)));G3_TRY(g3_name(T::BSpline(s)));G3_TRY(g3_ax1(T::AxeOfRevolution(s)));G3_TRY(g3_dir(T::Direction(s)));
 G3_TRY(g3_name(T::BasisCurve(s)));G3_TRY(g3_name(T::BasisSurface(s)));G3_TRY(g3_real(T::OffsetValue(s)));
 G3_TRY(g3_int(T::NbSamplesU(s)));G3_TRY(g3_int(T::NbSamplesV(s)));
 for(size_t i=0;i+1<ranges.size();i+=2){G3_TRY(g3_int(T::NbSamplesU(s,ranges[i],ranges[i+1])));G3_TRY(g3_int(T::NbSamplesV(s,ranges[i],ranges[i+1])));}
 for(double t:{Precision::Angular(),1e-3,0.5}){G3_TRY(g3_flag(T::IsSurfG1(s,true,t)));G3_TRY(g3_flag(T::IsSurfG1(s,false,t)));}
 std::cout<<'\n';
}
