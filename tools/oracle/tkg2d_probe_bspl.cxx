// TKG2d oracle: Bezier and B-spline curve edition and queries, and evaluation
// representations (tkg2d_probe.cxx).
// The deprecated copying getters are part of the public API under test.
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
// The deprecated copying getters, with matching and mismatching destinations.
void g2_bezier_arrays(const occ::handle<Geom2d_BezierCurve>& b){
 int n=b->NbPoles();
 for(int lower:{1,0}){NCollection_Array1<gp_Pnt2d> p(lower,lower+n-1);G2_TRY(b->Poles(p);for(const gp_Pnt2d& x:p)g2_pnt(x));
  NCollection_Array1<double> w(lower,lower+n-1);G2_TRY(b->Weights(w);for(double x:w)g2_real(x));}
 NCollection_Array1<gp_Pnt2d> p(1,n+1);G2_TRY(b->Poles(p));NCollection_Array1<double> w(1,n+1);G2_TRY(b->Weights(w));
}
void g2_bezier_state(const occ::handle<Geom2d_BezierCurve>& b,const std::vector<double>& u){
 g2_int(b->NbPoles());g2_int(b->Degree());g2_flag(b->IsRational());G2_TRY(g2_flag(b->IsClosed()));
 for(int i=0;i<=b->NbPoles()+1;++i){G2_TRY(g2_pnt(b->Pole(i)));G2_TRY(g2_real(b->Weight(i)));}
 for(double w:b->WeightsArray())g2_real(w);g2_flag(b->Weights()!=nullptr);
 for(double k:b->Knots())g2_real(k);for(int m:b->Multiplicities())g2_int(m);for(double k:b->KnotSequence())g2_real(k);
 g2_pnt(b->StartPoint());g2_pnt(b->EndPoint());double tol=0;b->Resolution(1e-7,tol);g2_real(tol);b->Resolution(2.5,tol);g2_real(tol);
 g2_bezier_arrays(b);g2_summary(b,u);g2_dump(b,-1);
}
// Actions: 1 Increase(d) 2 InsertPoleAfter(i,P,w) 3 InsertPoleBefore(i,P,w) 4 RemovePole(i) 5 Reverse
// 6 Segment(u1,u2) 7 SetPole(i,P) 8 SetPole(i,P,w) 9 SetWeight(i,w) 10 Transform.
void g2_bezier_action(const occ::handle<Geom2d_BezierCurve>& b,int action,G2Reader& r){
 int i=r.integer();gp_Pnt2d p=r.pnt();double w=r.real(),a=r.real(),z=r.real();gp_Trsf2d t=g2_trsf(r);
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
}
void g2_bezier_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_BezierCurve> b=occ::down_cast<Geom2d_BezierCurve>(c);int count=r.integer();
 g2_bezier_state(b,u);
 for(int k=0;k<count;++k){int action=r.integer();G2_TRY(g2_bezier_action(b,action,r));g2_bezier_state(b,u);}
 g2_bezier_state(new Geom2d_BezierCurve(*b),u);g2_int(Geom2d_BezierCurve::MaxDegree());std::cout<<'\n';
}
// The deprecated copying getters, with matching and mismatching destinations.
void g2_bspline_arrays(const occ::handle<Geom2d_BSplineCurve>& b){
 int nk=b->NbKnots(),np=b->NbPoles(),nf=b->KnotSequence().Length();
 for(int lower:{1,2,0}){NCollection_Array1<double> k(lower,nk);G2_TRY(b->Knots(k);for(double x:k)g2_real(x));
  NCollection_Array1<double> f(lower,nf);G2_TRY(b->KnotSequence(f);for(double x:f)g2_real(x));}
 for(int extra:{0,1}){NCollection_Array1<int> m(1,nk+extra);G2_TRY(b->Multiplicities(m);for(int x:m)g2_int(x));
  NCollection_Array1<gp_Pnt2d> p(1,np+extra);G2_TRY(b->Poles(p);for(const gp_Pnt2d& x:p)g2_pnt(x));
  NCollection_Array1<double> w(0,np-1+extra);G2_TRY(b->Weights(w);for(double x:w)g2_real(x));}
}
void g2_bspline_state(const occ::handle<Geom2d_BSplineCurve>& b,const std::vector<double>& u){
 g2_int(b->Degree());g2_flag(b->IsPeriodic());g2_flag(b->IsRational());g2_int(b->KnotDistribution());g2_int(b->Continuity());
 g2_int(b->NbKnots());g2_int(b->NbPoles());G2_TRY(g2_int(b->FirstUKnotIndex()));G2_TRY(g2_int(b->LastUKnotIndex()));
 for(int i=0;i<=b->NbKnots()+1;++i){G2_TRY(g2_real(b->Knot(i)));G2_TRY(g2_int(b->Multiplicity(i)));}
 for(int i=0;i<=b->NbPoles()+1;++i){G2_TRY(g2_pnt(b->Pole(i)));G2_TRY(g2_real(b->Weight(i)));}
 for(double k:b->KnotSequence())g2_real(k);for(double w:b->WeightsArray())g2_real(w);g2_flag(b->Weights()!=nullptr);
 G2_TRY(g2_pnt(b->StartPoint()));G2_TRY(g2_pnt(b->EndPoint()));double tol=0;G2_TRY(b->Resolution(1e-7,tol);g2_real(tol));
 G2_TRY(g2_flag(b->IsG1(b->FirstParameter(),b->LastParameter(),1e-7)));G2_TRY(g2_flag(b->IsG1(-1e9,1e9,0.5)));
 g2_bspline_arrays(b);
 for(double v:u){
  double n=v;b->PeriodicNormalization(n);g2_real(n);
  for(int rep=0;rep<=1;++rep)for(double tolerance:{0.0,1e-9,0.25}){int i1=-7,i2=-7;G2_TRY(b->LocateU(v,tolerance,i1,i2,rep==1);g2_int(i1);g2_int(i2));}
  for(int k1=b->FirstUKnotIndex();k1<b->LastUKnotIndex();++k1){int k2=k1+1;
   G2_TRY(g2_pnt(b->LocalValue(v,k1,k2)));
   G2_TRY(gp_Pnt2d p;gp_Vec2d d1;b->LocalD1(v,k1,k2,p,d1);g2_pnt(p);g2_vec(d1));
   G2_TRY(gp_Pnt2d p;gp_Vec2d d1,d2;b->LocalD2(v,k1,k2,p,d1,d2);g2_pnt(p);g2_vec(d1);g2_vec(d2));
   G2_TRY(gp_Pnt2d p;gp_Vec2d d1,d2,d3;b->LocalD3(v,k1,k2,p,d1,d2,d3);g2_pnt(p);g2_vec(d1);g2_vec(d2);g2_vec(d3));
   for(int n2=1;n2<=4;++n2)G2_TRY(g2_vec(b->LocalDN(v,k1,k2,n2)));}
  G2_TRY(g2_pnt(b->LocalValue(v,1,1)));
 }
 g2_summary(b,u);g2_dump(b,-1);
}
// Actions: 1 IncreaseDegree(i) 2 IncreaseMultiplicity(i,j) 3 IncreaseMultiplicity(i,j,k) 4 IncrementMultiplicity(i,j,k)
// 5 InsertKnot(a,i,w) 6 InsertKnots({a,z},{i,j},w,flag) 7 RemoveKnot(i,j,w) 8 InsertPoleAfter(i,P,w) 9 InsertPoleBefore(i,P,w)
// 10 RemovePole(i) 11 Reverse 12 Segment(a,z,w) 13 SetKnot(i,a) 14 SetKnots(shifted by a) 15 SetKnot(i,a,j) 16 SetPeriodic
// 17 SetOrigin(i) 18 SetNotPeriodic 19 SetPole(i,P) 20 SetPole(i,P,w) 21 SetWeight(i,w) 22 MovePoint(a,P,i,j)
// 23 MovePointAndTangent(a,P,(z,w),1e-9,i,j) 24 Transform.
void g2_bspline_action(const occ::handle<Geom2d_BSplineCurve>& b,int action,G2Reader& r){
 int i=r.integer(),j=r.integer(),k=r.integer();gp_Pnt2d p=r.pnt();double w=r.real(),a=r.real(),z=r.real();bool flag=r.flag();
 gp_Trsf2d t=g2_trsf(r);
 if(action==1)b->IncreaseDegree(i);
 if(action==2)b->IncreaseMultiplicity(i,j);
 if(action==3)b->IncreaseMultiplicity(i,j,k);
 if(action==4)b->IncrementMultiplicity(i,j,k);
 if(action==5)b->InsertKnot(a,i,w);
 if(action==6){NCollection_Array1<double> ks(1,2);ks(1)=a;ks(2)=z;NCollection_Array1<int> ms(1,2);ms(1)=i;ms(2)=j;b->InsertKnots(ks,ms,w,flag);}
 if(action==7)g2_flag(b->RemoveKnot(i,j,w));
 if(action==8)b->InsertPoleAfter(i,p,w);
 if(action==9)b->InsertPoleBefore(i,p,w);
 if(action==10)b->RemovePole(i);
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
 if(action==22){int first=-7,last=-7;b->MovePoint(a,p,i,j,first,last);g2_int(first);g2_int(last);}
 if(action==23){int status=-7;b->MovePointAndTangent(a,p,gp_Vec2d(z,w),1e-9,i,j,status);g2_int(status);}
 if(action==24)b->Transform(t);
}
void g2_bspline_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_BSplineCurve> b=occ::down_cast<Geom2d_BSplineCurve>(c);int count=r.integer();
 g2_bspline_state(b,u);
 for(int k=0;k<count;++k){int action=r.integer();G2_TRY(g2_bspline_action(b,action,r));g2_bspline_state(b,u);}
 g2_bspline_state(new Geom2d_BSplineCurve(*b),u);g2_int(Geom2d_BSplineCurve::MaxDegree());std::cout<<'\n';
}
// A descriptor over a representation curve: 0 Full, 1 DerivBounded, 2 Mapped (optional domain).
occ::handle<Geom2dEval_RepCurveDesc::Base> g2_desc(G2Reader& r,const occ::handle<Geom2d_Curve>& rep){
 int kind=r.integer(),order=r.integer();bool domain=r.flag();double first=r.real(),last=r.real(),scale=r.real(),offset=r.real();
 if(kind==0){occ::handle<Geom2dEval_RepCurveDesc::Full> d=new Geom2dEval_RepCurveDesc::Full();d->Representation=rep;return d;}
 if(kind==1){occ::handle<Geom2dEval_RepCurveDesc::DerivBounded> d=new Geom2dEval_RepCurveDesc::DerivBounded();d->Representation=rep;d->MaxDerivOrder=order;return d;}
 occ::handle<Geom2dEval_RepCurveDesc::Mapped> d=new Geom2dEval_RepCurveDesc::Mapped();d->Representation=rep;d->MaxDerivOrder=order;
 if(domain)d->Domain=Geom2dEval_RepCurveDesc::Domain1d{first,last};d->ParamMap.Scale=scale;d->ParamMap.Offset=offset;return d;
}
template<typename C> void g2_evalrep_on(const occ::handle<C>& c,const occ::handle<Geom2dEval_RepCurveDesc::Base>& d,const std::vector<double>& u,const gp_Trsf2d& t){
 G2_TRY(c->SetEvalRepresentation(d));g2_flag(c->HasEvalRepresentation());g2_summary(c,u);
 occ::handle<C> copy=occ::down_cast<C>(c->Copy());g2_flag(copy->HasEvalRepresentation());g2_summary(copy,u);
 G2_TRY(c->SetEvalRepresentation(new Geom2dEval_RepCurveDesc::Full()));
 occ::handle<Geom2dEval_RepCurveDesc::Full> self=new Geom2dEval_RepCurveDesc::Full();self->Representation=c;G2_TRY(c->SetEvalRepresentation(self));
 c->Reverse();g2_flag(c->HasEvalRepresentation());c->Transform(t);g2_flag(c->HasEvalRepresentation());g2_summary(c,u);
}
void g2_evalrep_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 occ::handle<Geom2d_Curve> rep=g2_curve(r);occ::handle<Geom2dEval_RepCurveDesc::Base> d=g2_desc(r,rep);gp_Trsf2d t=g2_trsf(r);
 if(auto b=occ::down_cast<Geom2d_BezierCurve>(c))g2_evalrep_on(b,d,u,t);
 if(auto b=occ::down_cast<Geom2d_BSplineCurve>(c))g2_evalrep_on(b,d,u,t);
 if(auto b=occ::down_cast<Geom2d_OffsetCurve>(c))g2_evalrep_on(b,d,u,t);
 std::cout<<'\n';
}
