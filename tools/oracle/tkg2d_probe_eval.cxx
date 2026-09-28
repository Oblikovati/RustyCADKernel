// TKG2d oracle: the Geom2dEval curves and the Geom2dHash hashers (tkg2d_probe.cxx).
// Eval curve kinds of g2_curve:
// 11 archimedean P D a b | 12 involute P D r | 13 logarithmic P D scale exponent |
// 14 sine P D amplitude omega phase |
// 15 tbezier rational lower np (x y)*np nw w*nw alpha |
// 16 aht rational lower degree alpha beta np (x y)*np nw w*nw.
occ::handle<Geom2d_Curve> g2_eval_bezier(int kind,G2Reader& r){
 bool rational=r.flag();int lower=r.integer(),degree=0;double alpha=0.,beta=0.;
 if(kind==16){degree=r.integer();alpha=r.real();beta=r.real();}
 int np=r.integer();NCollection_Array1<gp_Pnt2d> p(lower,lower+np-1);for(int i=lower;i<lower+np;++i)p(i)=r.pnt();
 int nw=r.integer();NCollection_Array1<double> w(lower,lower+nw-1);for(int i=lower;i<lower+nw;++i)w(i)=r.real();
 if(kind==15){double a=r.real();if(rational)return new Geom2dEval_TBezierCurve(p,w,a);return new Geom2dEval_TBezierCurve(p,a);}
 if(rational)return new Geom2dEval_AHTBezierCurve(p,w,degree,alpha,beta);
 return new Geom2dEval_AHTBezierCurve(p,degree,alpha,beta);
}
occ::handle<Geom2d_Curve> g2_eval_curve(int kind,G2Reader& r){
 if(kind>=15)return g2_eval_bezier(kind,r);
 gp_Ax2d a=r.ax();double x=r.real();
 if(kind==12)return new Geom2dEval_CircleInvoluteCurve(a,x);
 double y=r.real();
 if(kind==11)return new Geom2dEval_ArchimedeanSpiralCurve(a,x,y);
 if(kind==13)return new Geom2dEval_LogarithmicSpiralCurve(a,x,y);
 return new Geom2dEval_SineWaveCurve(a,x,y,r.real());
}
// Poles with their bounds; weights with their bounds, values only when stored (the
// non-rational AHT curve keeps one uninitialized weight).
void g2_eval_poles(const NCollection_Array1<gp_Pnt2d>& p){g2_int(p.Lower());g2_int(p.Length());for(const gp_Pnt2d& x:p)g2_pnt(x);}
void g2_eval_weights(const NCollection_Array1<double>& w,bool values){g2_int(w.Lower());g2_int(w.Length());if(values)for(double x:w)g2_real(x);}
void g2_eval_state(const occ::handle<Geom2d_Curve>& c){
 if(auto t=occ::down_cast<Geom2dEval_TBezierCurve>(c)){g2_int(t->NbPoles());g2_int(t->Order());g2_real(t->Alpha());
  g2_flag(t->IsRational());g2_eval_poles(t->Poles());g2_eval_weights(t->Weights(),true);G2_TRY(g2_pnt(t->StartPoint()));G2_TRY(g2_pnt(t->EndPoint()));}
 if(auto t=occ::down_cast<Geom2dEval_AHTBezierCurve>(c)){g2_int(t->NbPoles());g2_int(t->AlgDegree());g2_real(t->Alpha());
  g2_real(t->Beta());g2_flag(t->IsRational());g2_eval_poles(t->Poles());g2_eval_weights(t->Weights(),t->IsRational());
  G2_TRY(g2_pnt(t->StartPoint()));G2_TRY(g2_pnt(t->EndPoint()));}
 if(auto s=occ::down_cast<Geom2dEval_ArchimedeanSpiralCurve>(c)){g2_ax(s->Position());g2_real(s->InitialRadius());g2_real(s->GrowthRate());}
 if(auto s=occ::down_cast<Geom2dEval_CircleInvoluteCurve>(c)){g2_ax(s->Position());g2_real(s->Radius());}
 if(auto s=occ::down_cast<Geom2dEval_LogarithmicSpiralCurve>(c)){g2_ax(s->Position());g2_real(s->Scale());g2_real(s->GrowthExponent());}
 if(auto s=occ::down_cast<Geom2dEval_SineWaveCurve>(c)){g2_ax(s->Position());g2_real(s->Amplitude());g2_real(s->Omega());g2_real(s->Phase());}
}
// The class accessors of the curve and of its copy, the copy's summary, the high-order
// derivatives that g2_summary does not reach, and the depth-limited dumps.
void g2_eval_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;std::vector<double> u=g2_params(r);
 std::cout<<c->DynamicType()->Name()<<' ';g2_eval_state(c);occ::handle<Geom2d_Curve> copy=occ::down_cast<Geom2d_Curve>(c->Copy());
 std::cout<<copy->DynamicType()->Name()<<' ';g2_eval_state(copy);g2_summary(copy,u);
 for(double v:u)for(int n=6;n<=9;++n)G2_TRY(g2_vec(c->EvalDN(v,n)));
 g2_dump(c,0);g2_dump(c,1);g2_dump(c,2);std::cout<<'\n';
}
// Hashes print with an 'h' prefix so the NaN-tolerant comparator never reads them as doubles.
void g2_hash_value(size_t v){std::cout<<'h'<<v<<' ';}
// g2_hash comp hash n item*n; item: 0 <curve> | 1 (null handle) | 2 index (the same handle
// as an earlier item). Prints every CurveHasher hash, then equality for every ordered pair.
void g2_hash_case(std::istream& in){
 G2Reader r{in};double comp=r.real(),tol=r.real();int n=r.integer();std::vector<occ::handle<Geom2d_Curve>> curves;
 for(int i=0;i<n;++i){int mode=r.integer();occ::handle<Geom2d_Curve> c;
  if(mode==0){c=g2_built(r);if(c.IsNull())return;}
  if(mode==2)c=curves[r.integer()];
  curves.push_back(c);}
 Geom2dHash_CurveHasher h(comp,tol);
 for(const auto& c:curves)g2_hash_value(h(c));
 for(const auto& a:curves)for(const auto& b:curves)g2_flag(h(a,b));
 std::cout<<'\n';
}
// g2_hash_frame comp hash P1 P2 D1 D2 (frame P X Y)*2: the point, direction and gp_Ax22d
// hashers on raw values, including huge and non-finite coordinates.
void g2_hash_frame_case(std::istream& in){
 G2Reader r{in};double comp=r.real(),tol=r.real();gp_Pnt2d p1=r.pnt(),p2=r.pnt();gp_Dir2d d1=r.dir(),d2=r.dir();
 Geom2dHash_PointHasher points(comp,tol);Geom2dHash_DirectionHasher directions(comp,tol);
 g2_hash_value(points(p1));g2_hash_value(points(p2));g2_flag(points(p1,p2));
 g2_hash_value(directions(d1));g2_hash_value(directions(d2));g2_flag(directions(d1,d2));
 gp_Ax22d frames[2];
 for(gp_Ax22d& f:frames){gp_Pnt2d p=r.pnt();gp_Dir2d x=r.dir();gp_Dir2d y=r.dir();
  try{f=gp_Ax22d(p,x,y);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}}
 Geom2dHash_AxisPlacement axes(comp,tol);
 g2_hash_value(axes(frames[0]));g2_hash_value(axes(frames[1]));g2_flag(axes(frames[0],frames[1]));std::cout<<'\n';
}
