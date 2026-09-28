// TKG3d oracle: Geom_BezierSurface edition, queries and evaluation representations
// (tkg3d_probe.cxx). The deprecated copying getters are part of the public API under test.
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
// The deprecated copying getters with matching and mismatching destinations.
void g3_bzs_arrays(const occ::handle<Geom_BezierSurface>& b){
 int nu=b->NbUPoles(),nv=b->NbVPoles();
 for(int lower:{1,0}){NCollection_Array2<gp_Pnt> p(lower,lower+nu-1,lower,lower+nv-1);G3_TRY(b->Poles(p);for(const gp_Pnt& x:p)g3_pnt(x));
  NCollection_Array2<double> w(lower,lower+nu-1,lower,lower+nv-1);G3_TRY(b->Weights(w);for(double x:w)g3_real(x));}
 NCollection_Array2<gp_Pnt> p(1,nu+1,1,nv);G3_TRY(b->Poles(p));NCollection_Array2<double> w(1,nu,1,nv+1);G3_TRY(b->Weights(w));
}
// Every query of the class; `uv` are parameter pairs for the surface summary.
void g3_bzs_state(const occ::handle<Geom_BezierSurface>& b,const std::vector<double>& uv){
 g3_int(b->NbUPoles());g3_int(b->NbVPoles());g3_int(b->UDegree());g3_int(b->VDegree());
 g3_flag(b->IsURational());g3_flag(b->IsVRational());g3_flag(b->HasEvalRepresentation());
 for(int i=0;i<=b->NbUPoles()+1;++i)for(int j=0;j<=b->NbVPoles()+1;++j){
  G3_TRY(g3_pnt(b->Pole(i,j)));G3_TRY(g3_real(b->Weight(i,j)));}
 for(double w:b->WeightsArray())g3_real(w);g3_flag(b->Weights()!=nullptr);for(const gp_Pnt& x:b->Poles())g3_pnt(x);
 for(double k:b->UKnots())g3_real(k);for(double k:b->VKnots())g3_real(k);
 for(int m:b->UMultiplicities())g3_int(m);for(int m:b->VMultiplicities())g3_int(m);
 for(double k:b->UKnotSequence())g3_real(k);for(double k:b->VKnotSequence())g3_real(k);
 double ut=0,vt=0;b->Resolution(1e-7,ut,vt);g3_real(ut);g3_real(vt);b->Resolution(2.5,ut,vt);g3_real(ut);g3_real(vt);
 g3_bzs_arrays(b);g3_surface_summary(b,uv);g3_dump(b,-1);
}
// Reads a line of poles: lower, n, (x y z w)*n, weight lower shift; returns poles and weights.
struct G3Line{NCollection_Array1<gp_Pnt> p;NCollection_Array1<double> w;};
G3Line g3_bzs_line(G3Reader& r){
 int lower=r.integer(),n=r.integer();G3Line l{NCollection_Array1<gp_Pnt>(lower,lower+n-1),NCollection_Array1<double>()};
 std::vector<double> w(n);for(int k=0;k<n;++k){l.p(lower+k)=r.pnt();w[k]=r.real();}
 int shift=r.integer();l.w.Resize(lower+shift,lower+shift+n-1,false);for(int k=0;k<n;++k)l.w(lower+shift+k)=w[k];return l;
}
// Actions (arguments: i j P w a b c d line trsf): 1 ExchangeUV 2 Increase(i,j) 3/4 InsertPoleColAfter
// 5/6 InsertPoleColBefore 7/8 InsertPoleRowAfter 9/10 InsertPoleRowBefore (even: with weights)
// 11 RemovePoleCol(i) 12 RemovePoleRow(i) 13 Segment(a,b,c,d) 14 SetPole(i,j,P) 15 SetPole(i,j,P,w)
// 16/17 SetPoleCol(i,line) 18/19 SetPoleRow(i,line) (odd: with weights) 20 SetWeight(i,j,w)
// 21 SetWeightCol(i,weights) 22 SetWeightRow(i,weights) 23 UReverse 24 VReverse 25 Transform(t).
void g3_bzs_action(const occ::handle<Geom_BezierSurface>& b,int action,G3Reader& r){
 int i=r.integer(),j=r.integer();gp_Pnt p=r.pnt();double w=r.real(),a=r.real(),bb=r.real(),c=r.real(),d=r.real();
 G3Line l=g3_bzs_line(r);gp_Trsf t=g3_trsf(r);
 if(action==1)b->ExchangeUV();
 if(action==2)b->Increase(i,j);
 if(action==3)b->InsertPoleColAfter(i,l.p);
 if(action==4)b->InsertPoleColAfter(i,l.p,l.w);
 if(action==5)b->InsertPoleColBefore(i,l.p);
 if(action==6)b->InsertPoleColBefore(i,l.p,l.w);
 if(action==7)b->InsertPoleRowAfter(i,l.p);
 if(action==8)b->InsertPoleRowAfter(i,l.p,l.w);
 if(action==9)b->InsertPoleRowBefore(i,l.p);
 if(action==10)b->InsertPoleRowBefore(i,l.p,l.w);
 if(action==11)b->RemovePoleCol(i);
 if(action==12)b->RemovePoleRow(i);
 if(action==13)b->Segment(a,bb,c,d);
 if(action==14)b->SetPole(i,j,p);
 if(action==15)b->SetPole(i,j,p,w);
 if(action==16)b->SetPoleCol(i,l.p);
 if(action==17)b->SetPoleCol(i,l.p,l.w);
 if(action==18)b->SetPoleRow(i,l.p);
 if(action==19)b->SetPoleRow(i,l.p,l.w);
 if(action==20)b->SetWeight(i,j,w);
 if(action==21)b->SetWeightCol(i,l.w);
 if(action==22)b->SetWeightRow(i,l.w);
 if(action==23)b->UReverse();
 if(action==24)b->VReverse();
 if(action==25)b->Transform(t);
}
// <bezier spec> uv-list count (action args)*count: state after each action, then the copy.
void g3_bezier_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;std::vector<double> uv=r.reals();
 occ::handle<Geom_BezierSurface> b=occ::down_cast<Geom_BezierSurface>(s);int count=r.integer();
 g3_bzs_state(b,uv);
 for(int k=0;k<count;++k){int action=r.integer();G3_TRY(g3_bzs_action(b,action,r));g3_bzs_state(b,uv);}
 g3_bzs_state(new Geom_BezierSurface(*b),uv);g3_int(Geom_BezierSurface::MaxDegree());std::cout<<'\n';
}
// <bezier spec> uv-list <representation surface> <descriptor> count (action args)*count: which edits
// keep or clear the evaluation representation (re-set after each), evaluation through it, Copy.
void g3_bezier_surface_evalrep_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;std::vector<double> uv=r.reals();
 occ::handle<Geom_BezierSurface> b=occ::down_cast<Geom_BezierSurface>(s);
 occ::handle<Geom_Surface> rep=g3_surface(r);occ::handle<GeomEval_RepSurfaceDesc::Base> d=g3_surface_desc(r,rep);
 g3_flag(b->HasEvalRepresentation());G3_TRY(b->SetEvalRepresentation(d));g3_flag(b->HasEvalRepresentation());
 g3_flag(b->EvalRepresentation()==d);g3_surface_summary(b,uv);
 occ::handle<Geom_BezierSurface> c=occ::down_cast<Geom_BezierSurface>(b->Copy());g3_flag(c->HasEvalRepresentation());
 g3_flag(c->EvalRepresentation()==d);g3_surface_summary(c,uv);
 occ::handle<GeomEval_RepSurfaceDesc::Full> self=new GeomEval_RepSurfaceDesc::Full();self->Representation=b;
 G3_TRY(b->SetEvalRepresentation(self));g3_flag(b->EvalRepresentation()==d);
 occ::handle<GeomEval_RepSurfaceDesc::Full> none=new GeomEval_RepSurfaceDesc::Full();G3_TRY(b->SetEvalRepresentation(none));
 b->ClearEvalRepresentation();g3_flag(b->HasEvalRepresentation());
 int count=r.integer();
 for(int k=0;k<count;++k){int action=r.integer();G3_TRY(b->SetEvalRepresentation(d));
  G3_TRY(g3_bzs_action(b,action,r));g3_flag(b->HasEvalRepresentation());}
 G3_TRY(b->SetEvalRepresentation(d));g3_surface_summary(b,uv);std::cout<<'\n';
}
