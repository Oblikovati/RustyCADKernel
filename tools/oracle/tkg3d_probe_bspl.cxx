// TKG3d oracle: Bezier and B-spline curve edition and queries, and evaluation
// representations (tkg3d_probe.cxx).
// The deprecated copying getters are part of the public API under test.
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
// The deprecated copying getters, with matching and mismatching destinations.
void g3_bezier_arrays(const occ::handle<Geom_BezierCurve>& b){
 int n=b->NbPoles();
 for(int lower:{1,0}){NCollection_Array1<gp_Pnt> p(lower,lower+n-1);G3_TRY(b->Poles(p);for(const gp_Pnt& x:p)g3_pnt(x));
  NCollection_Array1<double> w(lower,lower+n-1);G3_TRY(b->Weights(w);for(double x:w)g3_real(x));}
 NCollection_Array1<gp_Pnt> p(1,n+1);G3_TRY(b->Poles(p));NCollection_Array1<double> w(1,n+1);G3_TRY(b->Weights(w));
}
void g3_bezier_state(const occ::handle<Geom_BezierCurve>& b,const std::vector<double>& u){
 g3_int(b->NbPoles());g3_int(b->Degree());g3_flag(b->IsRational());G3_TRY(g3_flag(b->IsClosed()));
 for(int i=0;i<=b->NbPoles()+1;++i){G3_TRY(g3_pnt(b->Pole(i)));G3_TRY(g3_real(b->Weight(i)));}
 for(double w:b->WeightsArray())g3_real(w);g3_flag(b->Weights()!=nullptr);for(const gp_Pnt& x:b->Poles())g3_pnt(x);
 for(double k:b->Knots())g3_real(k);for(int m:b->Multiplicities())g3_int(m);for(double k:b->KnotSequence())g3_real(k);
 g3_pnt(b->StartPoint());g3_pnt(b->EndPoint());double tol=0;b->Resolution(1e-7,tol);g3_real(tol);b->Resolution(2.5,tol);g3_real(tol);
 g3_bezier_arrays(b);g3_curve_summary(b,u);g3_dump(b,-1);
}
// Actions: 1 Increase(i) 2 InsertPoleAfter(i,P,w) 3 InsertPoleBefore(i,P,w) 4 RemovePole(i) 5 Reverse
// 6 Segment(a,z) 7 SetPole(i,P) 8 SetPole(i,P,w) 9 SetWeight(i,w) 10 Transform 11 InsertPoleAfter(i,P)
// 12 InsertPoleBefore(i,P).
void g3_bezier_action(const occ::handle<Geom_BezierCurve>& b,int action,G3Reader& r){
 int i=r.integer();gp_Pnt p=r.pnt();double w=r.real(),a=r.real(),z=r.real();gp_Trsf t=g3_trsf(r);
 if(action==1)b->Increase(i);
 if(action==2)b->InsertPoleAfter(i,p,w);
 if(action==3)b->InsertPoleBefore(i,p,w);
 if(action==4)b->RemovePole(i);
 if(action==5)b->Reverse();
 if(action==6)b->Segment(a,z);
 if(action==7)b->SetPole(i,p);
 if(action==8)b->SetPole(i,p,w);
 if(action==9)b->SetWeight(i,w);
 if(action==10)b->Transform(t);
 if(action==11)b->InsertPoleAfter(i,p);
 if(action==12)b->InsertPoleBefore(i,p);
}
void g3_bezier_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_BezierCurve> b=occ::down_cast<Geom_BezierCurve>(c);int count=r.integer();
 g3_bezier_state(b,u);
 for(int k=0;k<count;++k){int action=r.integer();G3_TRY(g3_bezier_action(b,action,r));g3_bezier_state(b,u);}
 g3_bezier_state(new Geom_BezierCurve(*b),u);g3_int(Geom_BezierCurve::MaxDegree());std::cout<<'\n';
}
// The deprecated copying getters, with matching and mismatching destinations.
void g3_bspline_arrays(const occ::handle<Geom_BSplineCurve>& b){
 int nk=b->NbKnots(),np=b->NbPoles(),nf=b->KnotSequence().Length();
 for(int lower:{1,2,0}){NCollection_Array1<double> k(lower,nk);G3_TRY(b->Knots(k);for(double x:k)g3_real(x));
  NCollection_Array1<double> f(lower,nf);G3_TRY(b->KnotSequence(f);for(double x:f)g3_real(x));}
 for(int extra:{0,1}){NCollection_Array1<int> m(1,nk+extra);G3_TRY(b->Multiplicities(m);for(int x:m)g3_int(x));
  NCollection_Array1<gp_Pnt> p(1,np+extra);G3_TRY(b->Poles(p);for(const gp_Pnt& x:p)g3_pnt(x));
  NCollection_Array1<double> w(0,np-1+extra);G3_TRY(b->Weights(w);for(double x:w)g3_real(x));}
}
// LocateU, the local evaluators between consecutive used knots and PeriodicNormalization at `v`.
void g3_bspline_local(const occ::handle<Geom_BSplineCurve>& b,double v){
 double n=v;b->PeriodicNormalization(n);g3_real(n);
 for(int rep=0;rep<=1;++rep)for(double tolerance:{0.0,1e-9,0.25}){int i1=-7,i2=-7;G3_TRY(b->LocateU(v,tolerance,i1,i2,rep==1);g3_int(i1);g3_int(i2));}
 for(int k1=b->FirstUKnotIndex();k1<b->LastUKnotIndex();++k1){int k2=k1+1;
  G3_TRY(g3_pnt(b->LocalValue(v,k1,k2)));
  G3_TRY(gp_Pnt p;b->LocalD0(v,k1,k2,p);g3_pnt(p));
  G3_TRY(gp_Pnt p;gp_Vec d1;b->LocalD1(v,k1,k2,p,d1);g3_pnt(p);g3_vec(d1));
  G3_TRY(gp_Pnt p;gp_Vec d1,d2;b->LocalD2(v,k1,k2,p,d1,d2);g3_pnt(p);g3_vec(d1);g3_vec(d2));
  G3_TRY(gp_Pnt p;gp_Vec d1,d2,d3;b->LocalD3(v,k1,k2,p,d1,d2,d3);g3_pnt(p);g3_vec(d1);g3_vec(d2);g3_vec(d3));
  for(int n2=1;n2<=4;++n2)G3_TRY(g3_vec(b->LocalDN(v,k1,k2,n2)));}
 G3_TRY(g3_pnt(b->LocalValue(v,1,1)));
}
void g3_bspline_state(const occ::handle<Geom_BSplineCurve>& b,const std::vector<double>& u){
 g3_int(b->Degree());g3_flag(b->IsPeriodic());g3_flag(b->IsRational());g3_int(b->KnotDistribution());g3_int(b->Continuity());
 g3_int(b->NbKnots());g3_int(b->NbPoles());G3_TRY(g3_int(b->FirstUKnotIndex()));G3_TRY(g3_int(b->LastUKnotIndex()));
 for(int i=0;i<=b->NbKnots()+1;++i){G3_TRY(g3_real(b->Knot(i)));G3_TRY(g3_int(b->Multiplicity(i)));}
 for(int i=0;i<=b->NbPoles()+1;++i){G3_TRY(g3_pnt(b->Pole(i)));G3_TRY(g3_real(b->Weight(i)));}
 for(double k:b->KnotSequence())g3_real(k);for(double w:b->WeightsArray())g3_real(w);g3_flag(b->Weights()!=nullptr);
 for(double k:b->Knots())g3_real(k);for(int m:b->Multiplicities())g3_int(m);for(const gp_Pnt& x:b->Poles())g3_pnt(x);
 G3_TRY(g3_pnt(b->StartPoint()));G3_TRY(g3_pnt(b->EndPoint()));double tol=0;G3_TRY(b->Resolution(1e-7,tol);g3_real(tol));
 G3_TRY(g3_flag(b->IsG1(b->FirstParameter(),b->LastParameter(),1e-7)));G3_TRY(g3_flag(b->IsG1(-1e9,1e9,0.5)));
 g3_flag(b->IsEqual(b,1e-7));G3_TRY(occ::handle<Geom_BSplineCurve> c=new Geom_BSplineCurve(*b);g3_flag(b->IsEqual(c,0.0));
  c->SetPole(1,c->Pole(1).Translated(gp_Vec(1e-9,0,0)));g3_flag(b->IsEqual(c,1e-10));g3_flag(b->IsEqual(c,1e-8)));
 g3_bspline_arrays(b);
 for(double v:u)g3_bspline_local(b,v);
 g3_curve_summary(b,u);g3_dump(b,-1);
}
// Actions: 1 IncreaseDegree(i) 2 IncreaseMultiplicity(i,j) 3 IncreaseMultiplicity(i,j,k) 4 IncrementMultiplicity(i,j,k)
// 5 InsertKnot(a,i,w,flag) 6 InsertKnots({a,z},{i,j},w,flag) 7 RemoveKnot(i,j,w) 8 SetOrigin(a,w)
// 11 Reverse 12 Segment(a,z,w) 13 SetKnot(i,a) 14 SetKnots(shifted by a) 15 SetKnot(i,a,j) 16 SetPeriodic
// 17 SetOrigin(i) 18 SetNotPeriodic 19 SetPole(i,P) 20 SetPole(i,P,w) 21 SetWeight(i,w) 22 MovePoint(a,P,i,j)
// 23 MovePointAndTangent(a,P,(z,w,a),1e-9,i,j) 24 Transform.
void g3_bspline_action(const occ::handle<Geom_BSplineCurve>& b,int action,G3Reader& r){
 int i=r.integer(),j=r.integer(),k=r.integer();gp_Pnt p=r.pnt();double w=r.real(),a=r.real(),z=r.real();bool flag=r.flag();
 gp_Trsf t=g3_trsf(r);
 if(action==1)b->IncreaseDegree(i);
 if(action==2)b->IncreaseMultiplicity(i,j);
 if(action==3)b->IncreaseMultiplicity(i,j,k);
 if(action==4)b->IncrementMultiplicity(i,j,k);
 if(action==5)b->InsertKnot(a,i,w,flag);
 if(action==6){NCollection_Array1<double> ks(1,2);ks(1)=a;ks(2)=z;NCollection_Array1<int> ms(1,2);ms(1)=i;ms(2)=j;b->InsertKnots(ks,ms,w,flag);}
 if(action==7)g3_flag(b->RemoveKnot(i,j,w));
 if(action==8)b->SetOrigin(a,w);
 if(action==11)b->Reverse();
 if(action==12)b->Segment(a,z,w);
 if(action==13)b->SetKnot(i,a);
 if(action==14){NCollection_Array1<double> ks(b->Knots());for(double& x:ks)x+=a;b->SetKnots(ks);}
 if(action==15)b->SetKnot(i,a,j);
 if(action==16)b->SetPeriodic();
 if(action==17)b->SetOrigin(i);
 if(action==18)b->SetNotPeriodic();
 if(action==19)b->SetPole(i,p);
 if(action==20)b->SetPole(i,p,w);
 if(action==21)b->SetWeight(i,w);
 if(action==22){int first=-7,last=-7;b->MovePoint(a,p,i,j,first,last);g3_int(first);g3_int(last);}
 if(action==23){int status=-7;b->MovePointAndTangent(a,p,gp_Vec(z,w,a),1e-9,i,j,status);g3_int(status);}
 if(action==24)b->Transform(t);
}
// The rational constructor with CheckRational false and true on the curve's own data.
void g3_bspline_rebuilt(const occ::handle<Geom_BSplineCurve>& b,const std::vector<double>& u){
 for(bool check:{false,true}){
  G3_TRY(occ::handle<Geom_BSplineCurve> c=new Geom_BSplineCurve(b->Poles(),b->WeightsArray(),b->Knots(),b->Multiplicities(),
   b->Degree(),b->IsPeriodic(),check);g3_flag(c->IsRational());g3_curve_summary(c,u);g3_dump(c,-1));}
}
void g3_bspline_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(c);int count=r.integer();
 g3_bspline_state(b,u);
 for(int k=0;k<count;++k){int action=r.integer();G3_TRY(g3_bspline_action(b,action,r));g3_bspline_state(b,u);}
 g3_bspline_state(new Geom_BSplineCurve(*b),u);g3_bspline_rebuilt(b,u);g3_int(Geom_BSplineCurve::MaxDegree());std::cout<<'\n';
}
// A descriptor over a representation curve: 0 Full, 1 DerivBounded, 2 Mapped (optional domain).
occ::handle<GeomEval_RepCurveDesc::Base> g3_desc(G3Reader& r,const occ::handle<Geom_Curve>& rep){
 int kind=r.integer(),order=r.integer();bool domain=r.flag();double first=r.real(),last=r.real(),scale=r.real(),offset=r.real();
 if(kind==0){occ::handle<GeomEval_RepCurveDesc::Full> d=new GeomEval_RepCurveDesc::Full();d->Representation=rep;return d;}
 if(kind==1){occ::handle<GeomEval_RepCurveDesc::DerivBounded> d=new GeomEval_RepCurveDesc::DerivBounded();d->Representation=rep;d->MaxDerivOrder=order;return d;}
 occ::handle<GeomEval_RepCurveDesc::Mapped> d=new GeomEval_RepCurveDesc::Mapped();d->Representation=rep;d->MaxDerivOrder=order;
 if(domain)d->Domain=GeomEval_RepCurveDesc::Domain1d{first,last};d->ParamMap.Scale=scale;d->ParamMap.Offset=offset;return d;
}
// Set the descriptor, evaluate, copy, reject invalid descriptors, then the edits that
// clear it (or not) natively.
template<typename C> void g3_evalrep_on(const occ::handle<C>& c,const occ::handle<GeomEval_RepCurveDesc::Base>& d,const std::vector<double>& u,const gp_Trsf& t){
 G3_TRY(c->SetEvalRepresentation(d));g3_flag(c->HasEvalRepresentation());g3_curve_summary(c,u);
 occ::handle<C> copy=occ::down_cast<C>(c->Copy());g3_flag(copy->HasEvalRepresentation());g3_curve_summary(copy,u);
 G3_TRY(c->SetEvalRepresentation(new GeomEval_RepCurveDesc::Full()));
 occ::handle<GeomEval_RepCurveDesc::Full> self=new GeomEval_RepCurveDesc::Full();self->Representation=c;G3_TRY(c->SetEvalRepresentation(self));
 c->Reverse();g3_flag(c->HasEvalRepresentation());g3_curve_summary(c,u);c->Transform(t);g3_flag(c->HasEvalRepresentation());
 g3_curve_summary(c,u);c->ClearEvalRepresentation();g3_flag(c->HasEvalRepresentation());
}
void g3_evalrep_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_Curve> rep=g3_curve(r);occ::handle<GeomEval_RepCurveDesc::Base> d=g3_desc(r,rep);gp_Trsf t=g3_trsf(r);
 if(auto b=occ::down_cast<Geom_BezierCurve>(c))g3_evalrep_on(b,d,u,t);
 if(auto b=occ::down_cast<Geom_BSplineCurve>(c))g3_evalrep_on(b,d,u,t);
 std::cout<<'\n';
}
// B-spline edits after a descriptor is set: whether each one keeps the descriptor.
void g3_evalrep_edit_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 occ::handle<Geom_Curve> rep=g3_curve(r);occ::handle<GeomEval_RepCurveDesc::Base> d=g3_desc(r,rep);
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(c);int count=r.integer();
 for(int k=0;k<count;++k){int action=r.integer();G3_TRY(b->SetEvalRepresentation(d));
  G3_TRY(g3_bspline_action(b,action,r));g3_flag(b->HasEvalRepresentation());g3_curve_summary(b,u);}
 std::cout<<'\n';
}
