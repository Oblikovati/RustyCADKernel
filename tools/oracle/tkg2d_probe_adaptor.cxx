// TKG2d oracle: Adaptor2d_Curve2d, Adaptor2d_Line2d, Adaptor2d_OffsetCurve,
// Geom2dAdaptor_Curve and Geom2dAdaptor::MakeCurve (tkg2d_probe.cxx).
// An adaptor specification, read recursively:
// 0 line P D u1 u2 | 1 default line Load(gp_Lin2d P D) | 2 default line Load(gp_Lin2d P D, u1, u2) |
// 3 Geom2dAdaptor_Curve(C) <curve> | 4 Geom2dAdaptor_Curve(C, u1, u2) <curve> |
// 5 Adaptor2d_OffsetCurve(A) <adaptor> | 6 Adaptor2d_OffsetCurve(A, d) <adaptor> |
// 7 Adaptor2d_OffsetCurve(A, d, w1, w2) <adaptor> | 8 Adaptor2d_Curve2d (the base class).
occ::handle<Adaptor2d_Curve2d> a2_adaptor(G2Reader& r){
 int kind=r.integer();
 if(kind<=2){gp_Pnt2d p=r.pnt();gp_Dir2d d=r.dir();
  if(kind==0){double u1=r.real();return new Adaptor2d_Line2d(p,d,u1,r.real());}
  occ::handle<Adaptor2d_Line2d> l=new Adaptor2d_Line2d();
  if(kind==1)l->Load(gp_Lin2d(p,d));else{double u1=r.real();l->Load(gp_Lin2d(p,d),u1,r.real());}
  return l;}
 if(kind==3)return new Geom2dAdaptor_Curve(g2_curve(r));
 if(kind==4){double u1=r.real(),u2=r.real();return new Geom2dAdaptor_Curve(g2_curve(r),u1,u2);}
 if(kind==5)return new Adaptor2d_OffsetCurve(a2_adaptor(r));
 if(kind==6){double d=r.real();return new Adaptor2d_OffsetCurve(a2_adaptor(r),d);}
 if(kind==7){double d=r.real(),w1=r.real(),w2=r.real();return new Adaptor2d_OffsetCurve(a2_adaptor(r),d,w1,w2);}
 if(kind==8)return new Adaptor2d_Curve2d();
 throw Standard_ProgramError("unknown adaptor kind");
}
occ::handle<Adaptor2d_Curve2d> a2_built(G2Reader& r){
 try{return a2_adaptor(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return nullptr;}
}
// NbIntervals and Intervals for every continuity, into an array of NbIntervals+1 values.
void a2_intervals(const occ::handle<Adaptor2d_Curve2d>& a){
 for(int s=0;s<=6;++s){int n=0;
  try{n=a->NbIntervals(GeomAbs_Shape(s));g2_int(n);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';continue;}
  NCollection_Array1<double> t(1,n+1);
  G2_TRY(a->Intervals(t,GeomAbs_Shape(s));g2_int(t.Lower());g2_int(t.Upper());for(double v:t)g2_real(v));}
}
void a2_kinds(const occ::handle<Adaptor2d_Curve2d>& a){
 G2_TRY(g2_int(a->GetType()));G2_TRY(g2_ax(a->Line().Position()));G2_TRY(gp_Circ2d c=a->Circle();g2_ax22(c.Axis());g2_real(c.Radius()));
 G2_TRY(gp_Elips2d e=a->Ellipse();g2_ax22(e.Axis());g2_real(e.MajorRadius());g2_real(e.MinorRadius()));
 G2_TRY(gp_Hypr2d h=a->Hyperbola();g2_ax22(h.Axis());g2_real(h.MajorRadius());g2_real(h.MinorRadius()));
 G2_TRY(gp_Parab2d p=a->Parabola();g2_ax22(p.Axis());g2_real(p.Focal()));
 G2_TRY(g2_int(a->Degree()));G2_TRY(g2_flag(a->IsRational()));G2_TRY(g2_int(a->NbPoles()));G2_TRY(g2_int(a->NbKnots()));
 G2_TRY(g2_int(a->NbSamples()));
 G2_TRY(occ::handle<Geom2d_BezierCurve> b=a->Bezier();g2_flag(b.IsNull());if(!b.IsNull())g2_int(b->NbPoles()));
 G2_TRY(occ::handle<Geom2d_BSplineCurve> b=a->BSpline();g2_flag(b.IsNull());if(!b.IsNull())g2_int(b->NbPoles()));
}
void a2_evaluations(const occ::handle<Adaptor2d_Curve2d>& a,double u){
 G2_TRY(g2_pnt(a->EvalD0(u)));
 G2_TRY(auto e=a->EvalD1(u);g2_pnt(e.Point);g2_vec(e.D1));
 G2_TRY(auto e=a->EvalD2(u);g2_pnt(e.Point);g2_vec(e.D1);g2_vec(e.D2));
 G2_TRY(auto e=a->EvalD3(u);g2_pnt(e.Point);g2_vec(e.D1);g2_vec(e.D2);g2_vec(e.D3));
 for(int n=0;n<=5;++n)G2_TRY(g2_vec(a->EvalDN(u,n)));
 gp_Pnt2d p;gp_Vec2d v1,v2,v3;
 G2_TRY(g2_pnt(a->Value(u)));G2_TRY(a->D0(u,p);g2_pnt(p));G2_TRY(a->D1(u,p,v1);g2_pnt(p);g2_vec(v1));
 G2_TRY(a->D2(u,p,v1,v2);g2_pnt(p);g2_vec(v1);g2_vec(v2));G2_TRY(a->D3(u,p,v1,v2,v3);g2_pnt(p);g2_vec(v1);g2_vec(v2);g2_vec(v3));
 G2_TRY(g2_vec(a->DN(u,2)));
}
// The adaptor-level summary: bounds, continuity and intervals, closure, periodicity,
// resolutions, kind queries, then the evaluators at every parameter.
void a2_summary(const occ::handle<Adaptor2d_Curve2d>& a,const std::vector<double>& params){
 std::cout<<a->DynamicType()->Name()<<' ';G2_TRY(g2_real(a->FirstParameter()));G2_TRY(g2_real(a->LastParameter()));
 G2_TRY(g2_int(a->Continuity()));a2_intervals(a);G2_TRY(g2_flag(a->IsClosed()));G2_TRY(g2_flag(a->IsPeriodic()));
 G2_TRY(g2_real(a->Period()));for(double r:{1e-7,1e-3,0.5,10.0})G2_TRY(g2_real(a->Resolution(r)));a2_kinds(a);
 for(double u:params)a2_evaluations(a,u);
}
void a2_make_curve(const occ::handle<Adaptor2d_Curve2d>& a,const std::vector<double>& params){
 try{occ::handle<Geom2d_Curve> c=Geom2dAdaptor::MakeCurve(*a);g2_flag(c.IsNull());
  if(!c.IsNull()){std::cout<<c->DynamicType()->Name()<<' ';g2_summary(c,params);}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
// Summary, MakeCurve, Trim and ShallowCopy of one adaptor.
void a2_adaptor_case(std::istream& in){
 G2Reader r{in};occ::handle<Adaptor2d_Curve2d> a=a2_built(r);if(a.IsNull())return;std::vector<double> u=r.reals();
 double t1=r.real(),t2=r.real();
 a2_summary(a,u);a2_make_curve(a,u);
 G2_TRY(occ::handle<Adaptor2d_Curve2d> t=a->Trim(t1,t2,1e-7);a2_summary(t,u);a2_make_curve(t,u));
 G2_TRY(occ::handle<Adaptor2d_Curve2d> c=a->ShallowCopy();a2_summary(c,u));std::cout<<'\n';
}
// The members safe on an empty Geom2dAdaptor_Curve (the others dereference the null curve).
void a2_empty_geom(const occ::handle<Geom2dAdaptor_Curve>& a){
 g2_flag(a->IsInitialized());g2_flag(a->Curve().IsNull());g2_real(a->FirstParameter());g2_real(a->LastParameter());
 g2_int(a->GetType());G2_TRY(g2_int(a->Continuity()));a2_intervals(a);for(double r:{1e-3,4.0})G2_TRY(g2_real(a->Resolution(r)));
 G2_TRY(g2_ax(a->Line().Position()));G2_TRY(g2_ax22(a->Circle().Axis()));G2_TRY(g2_int(a->Degree()));G2_TRY(g2_flag(a->IsRational()));
 G2_TRY(g2_int(a->NbPoles()));G2_TRY(g2_int(a->NbKnots()));g2_flag(a->Bezier().IsNull());g2_flag(a->BSpline().IsNull());
 G2_TRY(a->Trim(0.,1.,1e-7));G2_TRY(g2_real(a->ShallowCopy()->LastParameter()));a2_make_curve(a,{});
}
// Load sequences of Geom2dAdaptor_Curve: whole curve, the same curve on a range (cache
// reset), a failing range, another curve, Reset.
void a2_load_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c1=g2_built(r);if(c1.IsNull())return;occ::handle<Geom2d_Curve> c2=g2_curve(r);
 std::vector<double> u=r.reals();double u1=r.real(),u2=r.real();
 occ::handle<Geom2dAdaptor_Curve> a=new Geom2dAdaptor_Curve();a2_empty_geom(a);
 G2_TRY(a->Load(occ::handle<Geom2d_Curve>()));G2_TRY(a->Load(occ::handle<Geom2d_Curve>(),0.,1.));
 G2_TRY(a->Load(c1);std::cout<<a->Curve()->DynamicType()->Name()<<' ';a2_summary(a,u));
 G2_TRY(a->Load(c1,u1,u2);a2_summary(a,u));G2_TRY(a->Load(c1,u2+1.,u2));g2_real(a->FirstParameter());g2_real(a->LastParameter());
 G2_TRY(a->Load(c2,u1,u2);std::cout<<a->Curve()->DynamicType()->Name()<<' ';a2_summary(a,u));
 a->Reset();a2_empty_geom(a);G2_TRY(a->Load(c2);a2_summary(a,u));std::cout<<'\n';
}
// Load members of Adaptor2d_OffsetCurve, and the members safe without a basis.
void a2_offset_load_case(std::istream& in){
 G2Reader r{in};occ::handle<Adaptor2d_Curve2d> b1=a2_built(r);if(b1.IsNull())return;occ::handle<Adaptor2d_Curve2d> b2=a2_adaptor(r);
 std::vector<double> u=r.reals();double d=r.real(),w1=r.real(),w2=r.real();
 occ::handle<Adaptor2d_OffsetCurve> e=new Adaptor2d_OffsetCurve();g2_real(e->Offset());g2_real(e->FirstParameter());
 g2_real(e->LastParameter());g2_flag(e->Curve().IsNull());g2_real(e->Resolution(d));g2_real(e->ShallowCopy()->LastParameter());
 G2_TRY(g2_real(e->Trim(w1,w2,1e-7)->FirstParameter()));e->Load(d,w1,w2);g2_real(e->Offset());g2_real(e->LastParameter());
 occ::handle<Adaptor2d_OffsetCurve> o;
 try{o=new Adaptor2d_OffsetCurve(b1,d);}catch(const Standard_Failure& x){std::cout<<x.ExceptionType()<<'\n';return;}
 a2_summary(o,u);
 o->Load(b2);g2_real(o->Offset());g2_real(o->FirstParameter());g2_real(o->LastParameter());g2_flag(o->Curve()==b2);a2_summary(o,u);
 G2_TRY(o->Load(d));g2_real(o->Offset());g2_real(o->FirstParameter());g2_real(o->LastParameter());a2_summary(o,u);
 o->Load(-d,w1,w2);a2_summary(o,u);std::cout<<'\n';
}
// Sets (or, for a null descriptor, clears) the evaluation representation of a curve.
void a2_set_rep(const occ::handle<Geom2d_Curve>& c,const occ::handle<Geom2dEval_RepCurveDesc::Base>& d){
 if(auto b=occ::down_cast<Geom2d_BezierCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
 if(auto b=occ::down_cast<Geom2d_BSplineCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
 if(auto b=occ::down_cast<Geom2d_OffsetCurve>(c)){if(d.IsNull())b->ClearEvalRepresentation();else b->SetEvalRepresentation(d);}
}
// An adaptor on a curve with an evaluation representation. The adaptor records the
// representation at load time: clearing it later still routes evaluation to the curve,
// and reloading the same curve keeps the record, while a copy is reclassified.
void a2_evalrep_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;occ::handle<Geom2d_Curve> rep=g2_curve(r);
 occ::handle<Geom2dEval_RepCurveDesc::Base> d=g2_desc(r,rep);std::vector<double> u=r.reals();double u1=r.real(),u2=r.real();
 G2_TRY(a2_set_rep(c,d));occ::handle<Geom2dAdaptor_Curve> a=new Geom2dAdaptor_Curve(c,u1,u2);a2_summary(a,u);
 a2_set_rep(c,nullptr);a2_summary(a,u);a->Load(c,u1,u2);a2_summary(a,u);
 a->Load(occ::down_cast<Geom2d_Curve>(c->Copy()),u1,u2);a2_summary(a,u);std::cout<<'\n';
}
