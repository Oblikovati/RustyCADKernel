// TKG3d oracle: the Adaptor3d_Curve and Adaptor3d_Surface base classes, GeomAdaptor_Curve,
// GeomAdaptor_Surface, the swept and transformed surface adaptors and
// GeomAdaptor::MakeCurve/MakeSurface (tkg3d_probe.cxx).
// NbIntervals and Intervals for every continuity, into an array of NbIntervals+1 values.
void a3_curve_intervals(const occ::handle<Adaptor3d_Curve>& a){
 for(int s=0;s<=6;++s){int n=0;
  try{n=a->NbIntervals(GeomAbs_Shape(s));g3_int(n);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';continue;}
  NCollection_Array1<double> t(1,n+1);
  G3_TRY(a->Intervals(t,GeomAbs_Shape(s));for(double v:t)g3_real(v));}
}
void a3_curve_kinds(const occ::handle<Adaptor3d_Curve>& a){
 G3_TRY(g3_int(a->GetType()));G3_TRY(g3_ax1(a->Line().Position()));G3_TRY(gp_Circ c=a->Circle();g3_ax2(c.Position());g3_real(c.Radius()));
 G3_TRY(gp_Elips e=a->Ellipse();g3_ax2(e.Position());g3_real(e.MajorRadius());g3_real(e.MinorRadius()));
 G3_TRY(gp_Hypr h=a->Hyperbola();g3_ax2(h.Position());g3_real(h.MajorRadius());g3_real(h.MinorRadius()));
 G3_TRY(gp_Parab p=a->Parabola();g3_ax2(p.Position());g3_real(p.Focal()));
 G3_TRY(g3_int(a->Degree()));G3_TRY(g3_flag(a->IsRational()));G3_TRY(g3_int(a->NbPoles()));G3_TRY(g3_int(a->NbKnots()));
 G3_TRY(occ::handle<Geom_BezierCurve> b=a->Bezier();g3_name(b);if(!b.IsNull())g3_int(b->NbPoles()));
 G3_TRY(occ::handle<Geom_BSplineCurve> b=a->BSpline();g3_name(b);if(!b.IsNull())g3_int(b->NbPoles()));
 G3_TRY(occ::handle<Geom_OffsetCurve> o=a->OffsetCurve();g3_name(o);if(!o.IsNull())g3_real(o->Offset()));
}
void a3_curve_evaluations(const occ::handle<Adaptor3d_Curve>& a,double u){
 G3_TRY(g3_pnt(a->EvalD0(u)));
 G3_TRY(auto e=a->EvalD1(u);g3_pnt(e.Point);g3_vec(e.D1));
 G3_TRY(auto e=a->EvalD2(u);g3_pnt(e.Point);g3_vec(e.D1);g3_vec(e.D2));
 G3_TRY(auto e=a->EvalD3(u);g3_pnt(e.Point);g3_vec(e.D1);g3_vec(e.D2);g3_vec(e.D3));
 for(int n=0;n<=5;++n)G3_TRY(g3_vec(a->EvalDN(u,n)));
 gp_Pnt p;gp_Vec v1,v2,v3;
 G3_TRY(g3_pnt(a->Value(u)));G3_TRY(a->D0(u,p);g3_pnt(p));G3_TRY(a->D1(u,p,v1);g3_pnt(p);g3_vec(v1));
 G3_TRY(a->D2(u,p,v1,v2);g3_pnt(p);g3_vec(v1);g3_vec(v2));G3_TRY(a->D3(u,p,v1,v2,v3);g3_pnt(p);g3_vec(v1);g3_vec(v2);g3_vec(v3));
 G3_TRY(g3_vec(a->DN(u,2)));
}
// The adaptor-level summary of a curve: class, bounds, continuity and intervals, closure,
// periodicity, resolutions, kind queries, then the evaluators at every parameter.
void a3_curve_summary(const occ::handle<Adaptor3d_Curve>& a,const std::vector<double>& params){
 g3_name(a);G3_TRY(g3_real(a->FirstParameter()));G3_TRY(g3_real(a->LastParameter()));
 G3_TRY(g3_int(a->Continuity()));a3_curve_intervals(a);G3_TRY(g3_flag(a->IsClosed()));G3_TRY(g3_flag(a->IsPeriodic()));
 G3_TRY(g3_real(a->Period()));for(double r:{1e-7,1e-3,0.5,10.0})G3_TRY(g3_real(a->Resolution(r)));a3_curve_kinds(a);
 for(double u:params)a3_curve_evaluations(a,u);
}
void a3_make_curve(const occ::handle<Adaptor3d_Curve>& a,const std::vector<double>& params){
 try{occ::handle<Geom_Curve> c=GeomAdaptor::MakeCurve(*a);g3_name(c);if(!c.IsNull())g3_curve_summary(c,params);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
void a3_surface_intervals(const occ::handle<Adaptor3d_Surface>& a){
 for(int s=0;s<=6;++s){
  int n=0;
  try{n=a->NbUIntervals(GeomAbs_Shape(s));g3_int(n);NCollection_Array1<double> t(1,n+1);
   G3_TRY(a->UIntervals(t,GeomAbs_Shape(s));for(double v:t)g3_real(v));}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try{n=a->NbVIntervals(GeomAbs_Shape(s));g3_int(n);NCollection_Array1<double> t(1,n+1);
   G3_TRY(a->VIntervals(t,GeomAbs_Shape(s));for(double v:t)g3_real(v));}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
}
void a3_surface_kinds(const occ::handle<Adaptor3d_Surface>& a){
 G3_TRY(g3_int(a->GetType()));G3_TRY(g3_ax3(a->Plane().Position()));
 G3_TRY(gp_Cylinder c=a->Cylinder();g3_ax3(c.Position());g3_real(c.Radius()));
 G3_TRY(gp_Cone c=a->Cone();g3_ax3(c.Position());g3_real(c.RefRadius());g3_real(c.SemiAngle()));
 G3_TRY(gp_Sphere s=a->Sphere();g3_ax3(s.Position());g3_real(s.Radius()));
 G3_TRY(gp_Torus t=a->Torus();g3_ax3(t.Position());g3_real(t.MajorRadius());g3_real(t.MinorRadius()));
 G3_TRY(g3_int(a->UDegree()));G3_TRY(g3_int(a->NbUPoles()));G3_TRY(g3_int(a->VDegree()));G3_TRY(g3_int(a->NbVPoles()));
 G3_TRY(g3_int(a->NbUKnots()));G3_TRY(g3_int(a->NbVKnots()));G3_TRY(g3_flag(a->IsURational()));G3_TRY(g3_flag(a->IsVRational()));
 G3_TRY(occ::handle<Geom_BezierSurface> b=a->Bezier();g3_name(b);if(!b.IsNull())g3_int(b->NbUPoles()));
 G3_TRY(occ::handle<Geom_BSplineSurface> b=a->BSpline();g3_name(b);if(!b.IsNull())g3_int(b->NbUPoles()));
 G3_TRY(g3_ax1(a->AxeOfRevolution()));G3_TRY(g3_dir(a->Direction()));
 G3_TRY(occ::handle<Adaptor3d_Curve> c=a->BasisCurve();g3_name(c);if(!c.IsNull()){G3_TRY(g3_real(c->FirstParameter());g3_real(c->LastParameter());g3_int(c->GetType()));});
 G3_TRY(occ::handle<Adaptor3d_Surface> s=a->BasisSurface();g3_name(s);G3_TRY(g3_real(s->FirstUParameter());g3_real(s->LastUParameter());
        g3_real(s->FirstVParameter());g3_real(s->LastVParameter());g3_int(s->GetType())));
 G3_TRY(g3_real(a->OffsetValue()));
}
void a3_surface_evaluations(const occ::handle<Adaptor3d_Surface>& a,double u,double v){
 G3_TRY(g3_pnt(a->EvalD0(u,v)));
 G3_TRY(auto r=a->EvalD1(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V));
 G3_TRY(auto r=a->EvalD2(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV));
 G3_TRY(auto r=a->EvalD3(u,v);g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV);
        g3_vec(r.D3U);g3_vec(r.D3V);g3_vec(r.D3UUV);g3_vec(r.D3UVV));
 for(int nu=0;nu<=3;++nu)for(int nv=0;nv<=3;++nv)if(nu+nv>0)G3_TRY(g3_vec(a->EvalDN(u,v,nu,nv)));
 gp_Pnt p;gp_Vec d1u,d1v,d2u,d2v,d2uv,d3u,d3v,d3uuv,d3uvv;
 G3_TRY(g3_pnt(a->Value(u,v)));G3_TRY(a->D0(u,v,p);g3_pnt(p));G3_TRY(a->D1(u,v,p,d1u,d1v);g3_pnt(p);g3_vec(d1u);g3_vec(d1v));
 G3_TRY(a->D2(u,v,p,d1u,d1v,d2u,d2v,d2uv);g3_pnt(p);g3_vec(d2u);g3_vec(d2v);g3_vec(d2uv));
 G3_TRY(a->D3(u,v,p,d1u,d1v,d2u,d2v,d2uv,d3u,d3v,d3uuv,d3uvv);g3_pnt(p);g3_vec(d3u);g3_vec(d3v);g3_vec(d3uuv);g3_vec(d3uvv));
 G3_TRY(g3_vec(a->DN(u,v,1,1)));
}
// The adaptor-level summary of a surface, as a3_curve_summary.
void a3_surface_summary(const occ::handle<Adaptor3d_Surface>& a,const std::vector<double>& uv){
 g3_name(a);G3_TRY(g3_real(a->FirstUParameter()));G3_TRY(g3_real(a->LastUParameter()));
 G3_TRY(g3_real(a->FirstVParameter()));G3_TRY(g3_real(a->LastVParameter()));
 G3_TRY(g3_int(a->UContinuity()));G3_TRY(g3_int(a->VContinuity()));a3_surface_intervals(a);
 G3_TRY(g3_flag(a->IsUClosed()));G3_TRY(g3_flag(a->IsVClosed()));G3_TRY(g3_flag(a->IsUPeriodic()));G3_TRY(g3_flag(a->IsVPeriodic()));
 G3_TRY(g3_real(a->UPeriod()));G3_TRY(g3_real(a->VPeriod()));
 for(double r:{1e-7,1e-3,0.5,10.0}){G3_TRY(g3_real(a->UResolution(r)));G3_TRY(g3_real(a->VResolution(r)));}
 a3_surface_kinds(a);for(size_t i=0;i+1<uv.size();i+=2)a3_surface_evaluations(a,uv[i],uv[i+1]);
}
void a3_make_surface(const occ::handle<Adaptor3d_Surface>& a,const std::vector<double>& uv){
 for(bool trim:{true,false}){
  try{occ::handle<Geom_Surface> s=GeomAdaptor::MakeSurface(*a,trim);g3_name(s);if(!s.IsNull())g3_surface_summary(s,uv);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
}
// Adaptor3d_Curve and Adaptor3d_Surface themselves: every member raises.
void a3_base_case(std::istream& in){
 G3Reader r{in};std::vector<double> u=r.reals();
 occ::handle<Adaptor3d_Curve> c=new Adaptor3d_Curve();a3_curve_summary(c,u);G3_TRY(c->ShallowCopy());G3_TRY(c->Trim(0.,1.,1e-7));
 a3_make_curve(c,u);
 occ::handle<Adaptor3d_Surface> s=new Adaptor3d_Surface();a3_surface_summary(s,u);G3_TRY(s->ShallowCopy());
 G3_TRY(s->UTrim(0.,1.,1e-7));G3_TRY(s->VTrim(0.,1.,1e-7));a3_make_surface(s,u);std::cout<<'\n';
}
// A curve adaptor: 0 GeomAdaptor_Curve(C) | 1 GeomAdaptor_Curve(C, u1, u2) <curve>.
occ::handle<GeomAdaptor_Curve> a3_geom_curve(G3Reader& r){
 int kind=r.integer();
 if(kind==0)return new GeomAdaptor_Curve(g3_curve(r));
 double u1=r.real(),u2=r.real();return new GeomAdaptor_Curve(g3_curve(r),u1,u2);
}
// <curve adaptor> params t1 t2: summary, MakeCurve, Trim and ShallowCopy.
void a3_curve_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Curve> a;
 try{a=a3_geom_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> u=r.reals();double t1=r.real(),t2=r.real();
 g3_name(a->Curve());a3_curve_summary(a,u);a3_make_curve(a,u);
 G3_TRY(occ::handle<Adaptor3d_Curve> t=a->Trim(t1,t2,1e-7);a3_curve_summary(t,u);a3_make_curve(t,u));
 G3_TRY(occ::handle<Adaptor3d_Curve> c=a->ShallowCopy();a3_curve_summary(c,u));std::cout<<'\n';
}
// The members safe on an empty GeomAdaptor_Curve (the others dereference the null curve).
void a3_empty_curve(const occ::handle<GeomAdaptor_Curve>& a){
 g3_name(a->Curve());g3_real(a->FirstParameter());g3_real(a->LastParameter());g3_int(a->GetType());
 G3_TRY(g3_int(a->Continuity()));a3_curve_intervals(a);for(double r:{1e-3,4.0})G3_TRY(g3_real(a->Resolution(r)));a3_curve_kinds(a);
 G3_TRY(a->Trim(0.,1.,1e-7));G3_TRY(g3_real(a->ShallowCopy()->LastParameter()));a3_make_curve(a,{});
}
// Load sequences: whole curve, the same curve on a range (cache reset), a failing range,
// another curve, Reset.
void a3_curve_load_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c1=g3_built_curve(r);if(c1.IsNull())return;occ::handle<Geom_Curve> c2=g3_curve(r);
 std::vector<double> u=r.reals();double u1=r.real(),u2=r.real();
 occ::handle<GeomAdaptor_Curve> a=new GeomAdaptor_Curve();a3_empty_curve(a);
 G3_TRY(a->Load(occ::handle<Geom_Curve>()));G3_TRY(a->Load(occ::handle<Geom_Curve>(),0.,1.));
 G3_TRY(a->Load(c1);g3_name(a->Curve());a3_curve_summary(a,u));
 G3_TRY(a->Load(c1,u1,u2);a3_curve_summary(a,u));G3_TRY(a->Load(c1,u2+1.,u2));g3_real(a->FirstParameter());g3_real(a->LastParameter());
 G3_TRY(a->Load(c2,u1,u2);g3_name(a->Curve());a3_curve_summary(a,u));
 a->Reset();a3_empty_curve(a);G3_TRY(a->Load(c2);a3_curve_summary(a,u));std::cout<<'\n';
}
void a3_set_curve_rep(const occ::handle<Geom_Curve>& c,const occ::handle<GeomEval_RepCurveDesc::Base>& d){
 if(auto b=occ::down_cast<Geom_BezierCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
 if(auto b=occ::down_cast<Geom_BSplineCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
 if(auto b=occ::down_cast<Geom_OffsetCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
}
// A curve adaptor on a curve with an evaluation representation, recorded at load time:
// clearing it later still routes evaluation to the curve until a new curve is loaded.
void a3_curve_evalrep_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;occ::handle<Geom_Curve> rep=g3_curve(r);
 occ::handle<GeomEval_RepCurveDesc::Base> d=g3_desc(r,rep);std::vector<double> u=r.reals();double u1=r.real(),u2=r.real();
 G3_TRY(a3_set_curve_rep(c,d));occ::handle<GeomAdaptor_Curve> a=new GeomAdaptor_Curve(c,u1,u2);a3_curve_summary(a,u);
 a3_set_curve_rep(c,nullptr);a3_curve_summary(a,u);a->Load(c,u1,u2);a3_curve_summary(a,u);
 a->Load(occ::down_cast<Geom_Curve>(c->Copy()),u1,u2);a3_curve_summary(a,u);std::cout<<'\n';
}
// A surface adaptor: 0 GeomAdaptor_Surface(S) | 1 (S, u1, u2, v1, v2, tolu, tolv) <surface>.
occ::handle<GeomAdaptor_Surface> a3_geom_surface(G3Reader& r){
 int kind=r.integer();
 if(kind==0)return new GeomAdaptor_Surface(g3_surface(r));
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tu=r.real(),tv=r.real();
 return new GeomAdaptor_Surface(g3_surface(r),u1,u2,v1,v2,tu,tv);
}
// <surface adaptor> uv u1 u2 v1 v2 tol: summary, MakeSurface, UTrim, VTrim and ShallowCopy.
void a3_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Surface> a;
 try{a=a3_geom_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> uv=r.reals();double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real();
 g3_name(a->Surface());g3_real(a->ToleranceU());g3_real(a->ToleranceV());
 double b[4];a->Bounds(b[0],b[1],b[2],b[3]);for(double x:b)g3_real(x);
 a3_surface_summary(a,uv);a3_make_surface(a,uv);
 G3_TRY(occ::handle<Adaptor3d_Surface> t=a->UTrim(u1,u2,tol);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=a->VTrim(v1,v2,tol);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> c=a->ShallowCopy();a3_surface_summary(c,uv));std::cout<<'\n';
}
// Load sequences of GeomAdaptor_Surface: null and inverted ranges, the whole surface,
// the same surface on other bounds (the data and caches are kept), another surface.
void a3_surface_load_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s1=g3_built_surface(r);if(s1.IsNull())return;occ::handle<Geom_Surface> s2=g3_surface(r);
 std::vector<double> uv=r.reals();double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real();
 occ::handle<GeomAdaptor_Surface> a=new GeomAdaptor_Surface();g3_name(a->Surface());
 g3_real(a->FirstUParameter());g3_real(a->LastVParameter());g3_int(a->GetType());a3_surface_kinds(a);
 G3_TRY(g3_int(a->UContinuity()));G3_TRY(g3_int(a->NbUIntervals(GeomAbs_C1)));G3_TRY(g3_real(a->UResolution(0.1)));
 G3_TRY(a->Load(occ::handle<Geom_Surface>()));G3_TRY(a->Load(occ::handle<Geom_Surface>(),0.,1.,0.,1.));
 G3_TRY(a->Load(s1);a3_surface_summary(a,uv));
 G3_TRY(a->Load(s1,u1,u2,v1,v2,tol,tol);a3_surface_summary(a,uv));
 G3_TRY(a->Load(s1,u2,u1,v1,v2));G3_TRY(a->Load(s1,u1,u2,v2+1.,v2));
 double b[4];a->Bounds(b[0],b[1],b[2],b[3]);for(double x:b)g3_real(x);g3_real(a->ToleranceU());
 G3_TRY(a->Load(s2,u1,u2,v1,v2,tol,0.);g3_name(a->Surface());a3_surface_summary(a,uv));std::cout<<'\n';
}
bool a3_set_surface_rep(const occ::handle<Geom_Surface>& s,const occ::handle<GeomEval_RepSurfaceDesc::Base>& d){
 if(auto b=occ::down_cast<Geom_BezierSurface>(s)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);return true;}
 if(auto b=occ::down_cast<Geom_BSplineSurface>(s)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);return true;}
 if(auto b=occ::down_cast<Geom_OffsetSurface>(s)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);return true;}
 if(auto b=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);return true;}
 if(auto b=occ::down_cast<Geom_SurfaceOfRevolution>(s)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);return true;}
 return false;
}
// A surface adaptor on a surface with an evaluation representation (recorded at load).
void a3_surface_evalrep_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;occ::handle<Geom_Surface> rep=g3_surface(r);
 occ::handle<GeomEval_RepSurfaceDesc::Base> d=g3_surface_desc(r,rep);std::vector<double> uv=r.reals();
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real();
 G3_TRY(g3_flag(a3_set_surface_rep(s,d)));occ::handle<GeomAdaptor_Surface> a=new GeomAdaptor_Surface(s,u1,u2,v1,v2,tol,tol);
 a3_surface_summary(a,uv);a3_set_surface_rep(s,nullptr);a3_surface_summary(a,uv);
 a->Load(occ::down_cast<Geom_Surface>(s->Copy()),u1,u2,v1,v2,tol,tol);a3_surface_summary(a,uv);std::cout<<'\n';
}
// The members of an extrusion without basis curve that do not dereference it.
void a3_empty_extrusion(const occ::handle<GeomAdaptor_SurfaceOfLinearExtrusion>& e){
 g3_real(e->FirstVParameter());g3_real(e->LastVParameter());g3_int(e->VContinuity());g3_int(e->NbVIntervals(GeomAbs_C2));
 g3_flag(e->IsVClosed());g3_flag(e->IsVPeriodic());G3_TRY(g3_real(e->VPeriod()));g3_real(e->VResolution(0.25));
 G3_TRY(e->Cone());G3_TRY(e->Sphere());G3_TRY(e->Torus());G3_TRY(e->AxeOfRevolution());G3_TRY(e->IsURational());
 G3_TRY(e->IsVRational());G3_TRY(e->Bezier());G3_TRY(e->BSpline());g3_dir(e->Direction());g3_name(e->BasisCurve());
 G3_TRY(g3_int(e->VDegree()));G3_TRY(g3_int(e->NbVPoles()));G3_TRY(g3_int(e->NbVKnots()));G3_TRY(e->BasisSurface());
 G3_TRY(g3_real(e->OffsetValue()));g3_name(e->Surface());
}
// <curve adaptor> uv D mode v1 v2 u1 u2: an extrusion built by mode (0 curve only, 1 curve
// and direction), then Load of the direction and of a trimmed curve, trims and copies.
void a3_extrusion_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Curve> c;
 try{c=a3_geom_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> uv=r.reals();gp_Dir d=r.dir();int mode=r.integer();double v1=r.real(),v2=r.real(),u1=r.real(),u2=r.real();
 occ::handle<GeomAdaptor_SurfaceOfLinearExtrusion> e=
   mode==0?new GeomAdaptor_SurfaceOfLinearExtrusion(c):new GeomAdaptor_SurfaceOfLinearExtrusion(c,d);
 // Without a direction the base evaluators dereference the null surface: no evaluation.
 g3_name(e->Surface());a3_surface_summary(e,mode==0?std::vector<double>():uv);G3_TRY(double b[4];e->Bounds(b[0],b[1],b[2],b[3]);for(double x:b)g3_real(x));
 e->Load(d);a3_surface_summary(e,uv);a3_make_surface(e,uv);
 G3_TRY(e->Load(c->Trim(u1,u2,1e-7));a3_surface_summary(e,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->UTrim(u1,u2,1e-7);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->VTrim(v1,v2,1e-7);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->ShallowCopy();a3_surface_summary(t,uv));
 occ::handle<GeomAdaptor_SurfaceOfLinearExtrusion> empty=new GeomAdaptor_SurfaceOfLinearExtrusion();
 a3_empty_extrusion(empty);a3_empty_extrusion(occ::down_cast<GeomAdaptor_SurfaceOfLinearExtrusion>(empty->ShallowCopy()));
 empty->Load(d);a3_empty_extrusion(empty);std::cout<<'\n';
}
// The members of a revolution without basis curve that do not dereference it.
void a3_empty_revolution(const occ::handle<GeomAdaptor_SurfaceOfRevolution>& e){
 g3_ax3(e->Axis());g3_real(e->FirstUParameter());g3_real(e->LastUParameter());g3_int(e->UContinuity());g3_int(e->NbUIntervals(GeomAbs_C2));
 NCollection_Array1<double> t(3,4);e->UIntervals(t,GeomAbs_C1);for(double v:t)g3_real(v);
 g3_flag(e->IsUClosed());g3_flag(e->IsUPeriodic());g3_real(e->UPeriod());g3_real(e->UResolution(0.25));
 G3_TRY(e->NbVKnots());G3_TRY(e->IsURational());G3_TRY(e->IsVRational());G3_TRY(e->Bezier());G3_TRY(e->BSpline());
 g3_ax1(e->AxeOfRevolution());g3_name(e->BasisCurve());G3_TRY(g3_int(e->UDegree()));G3_TRY(g3_int(e->NbUPoles()));
 G3_TRY(g3_int(e->NbUKnots()));G3_TRY(e->Direction());G3_TRY(e->BasisSurface());G3_TRY(g3_real(e->OffsetValue()));
 G3_TRY(e->UTrim(0.1,1.,1e-7));g3_name(e->Surface());
}
// <curve adaptor> uv P D mode v1 v2 u1 u2: a surface of revolution, as a3_extrusion_case.
void a3_revolution_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Curve> c;
 try{c=a3_geom_curve(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> uv=r.reals();gp_Ax1 ax=r.ax1();int mode=r.integer();double v1=r.real(),v2=r.real(),u1=r.real(),u2=r.real();
 occ::handle<GeomAdaptor_SurfaceOfRevolution> e;
 try{e=mode==0?new GeomAdaptor_SurfaceOfRevolution(c):new GeomAdaptor_SurfaceOfRevolution(c,ax);}
 catch(const Standard_Failure& x){std::cout<<x.ExceptionType()<<'\n';return;}
 // Without an axis GetType evaluates through the null base surface: no summary.
 g3_ax3(e->Axis());if(mode!=0)a3_surface_summary(e,uv);G3_TRY(e->Load(ax));g3_ax3(e->Axis());a3_surface_summary(e,uv);a3_make_surface(e,uv);
 G3_TRY(e->Load(c->Trim(v1,v2,1e-7));g3_ax3(e->Axis());a3_surface_summary(e,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->UTrim(u1,u2,1e-7);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->UTrim(0.,2.*M_PI,1e-7);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->VTrim(v1,v2,1e-7);a3_surface_summary(t,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> t=e->ShallowCopy();a3_surface_summary(t,uv));
 occ::handle<GeomAdaptor_SurfaceOfRevolution> empty=new GeomAdaptor_SurfaceOfRevolution();
 a3_empty_revolution(empty);a3_empty_revolution(occ::down_cast<GeomAdaptor_SurfaceOfRevolution>(empty->ShallowCopy()));std::cout<<'\n';
}
// <surface adaptor> uv trsf trsf2 tol: the transformed adaptor, its accessors, SetTrsf and
// Load, trims and copies.
void a3_transformed_accessors(const occ::handle<GeomAdaptor_TransformedSurface>& t){
 g3_flag(t->HasTrsf());g3_name(t->GeomSurfaceOriginal());g3_real(t->ToleranceU());g3_real(t->ToleranceV());
 G3_TRY(g3_name(t->GeomSurfaceTransformed());g3_flag(t->GeomSurfaceTransformed()==t->GeomSurfaceTransformed());
        g3_flag(t->GeomSurfaceTransformed()==t->GeomSurfaceOriginal()));
 G3_TRY(const GeomAdaptor_Surface& s=t->AdaptorSurfaceTransformed();g3_flag(&s==&t->AdaptorSurfaceOriginal());
        g3_real(s.FirstUParameter());g3_int(s.GetType()));
}
void a3_transformed_state(const occ::handle<GeomAdaptor_TransformedSurface>& t,const std::vector<double>& uv){
 a3_transformed_accessors(t);a3_surface_summary(t,uv);
}
void a3_transformed_case(std::istream& in){
 G3Reader r{in};occ::handle<GeomAdaptor_Surface> a;
 try{a=a3_geom_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 std::vector<double> uv=r.reals();gp_Trsf m=g3_trsf(r),m2=g3_trsf(r);double tol=r.real();
 double b[4];a->Bounds(b[0],b[1],b[2],b[3]);
 occ::handle<GeomAdaptor_TransformedSurface> t;
 try{t=new GeomAdaptor_TransformedSurface(a->Surface(),b[0],b[1],b[2],b[3],m,tol,tol);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 a3_transformed_state(t,uv);a3_make_surface(t,uv);
 G3_TRY(occ::handle<Adaptor3d_Surface> c=t->UTrim(b[0],(b[0]+b[1])/2.,tol);a3_surface_summary(c,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> c=t->VTrim(b[2],(b[2]+b[3])/2.,tol);a3_surface_summary(c,uv));
 G3_TRY(occ::handle<Adaptor3d_Surface> c=t->ShallowCopy();a3_surface_summary(c,uv));
 t->SetTrsf(m2);a3_transformed_state(t,uv);t->SetTrsf(gp_Trsf());a3_transformed_state(t,uv);
 G3_TRY(t->Load(a->Surface(),m);a3_transformed_state(t,uv));
 occ::handle<GeomAdaptor_TransformedSurface> empty=new GeomAdaptor_TransformedSurface();a3_transformed_accessors(empty);
 empty->SetTrsf(m);a3_transformed_accessors(empty);std::cout<<'\n';
}
