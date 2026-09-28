// TKG3d oracle: the GeomEval curves (tkg3d_probe.cxx). Curve kinds of g3_curve from 20:
// 20 sine F amplitude omega phase | 21 helix F radius pitch |
// 22 tbezier rational lower np (x y z)*np nw w*nw alpha |
// 23 aht rational lower degree alpha beta np (x y z)*np nw w*nw, where F is a gp_Ax2 (P N X).
#include <GeomEval_AHTBezierCurve.hxx>
#include <GeomEval_CircularHelixCurve.hxx>
#include <GeomEval_SineWaveCurve.hxx>
#include <GeomEval_TBezierCurve.hxx>
occ::handle<Geom_Curve> g3_eval_bezier_curve(int kind,G3Reader& r){
 bool rational=r.flag();int lower=r.integer(),degree=0;double alpha=0.,beta=0.;
 if(kind==23){degree=r.integer();alpha=r.real();beta=r.real();}
 int np=r.integer();NCollection_Array1<gp_Pnt> p(lower,lower+np-1);for(int i=lower;i<lower+np;++i)p(i)=r.pnt();
 int nw=r.integer();NCollection_Array1<double> w(lower,lower+nw-1);for(int i=lower;i<lower+nw;++i)w(i)=r.real();
 if(kind==22){double a=r.real();if(rational)return new GeomEval_TBezierCurve(p,w,a);return new GeomEval_TBezierCurve(p,a);}
 if(rational)return new GeomEval_AHTBezierCurve(p,w,degree,alpha,beta);
 return new GeomEval_AHTBezierCurve(p,degree,alpha,beta);
}
occ::handle<Geom_Curve> g3_eval_curve(int kind,G3Reader& r){
 if(kind==22||kind==23)return g3_eval_bezier_curve(kind,r);
 if(kind!=20&&kind!=21)throw Standard_ProgramError(("unknown eval curve kind "+std::to_string(kind)).c_str());
 gp_Ax2 f=r.ax2();double x=r.real(),y=r.real();
 if(kind==21)return new GeomEval_CircularHelixCurve(f,x,y);
 return new GeomEval_SineWaveCurve(f,x,y,r.real());
}
// Poles with their bounds; weights with their bounds, values only when stored (the
// non-rational AHT curve keeps one uninitialized weight).
void g3_eval_curve_poles(const NCollection_Array1<gp_Pnt>& p){g3_int(p.Lower());g3_int(p.Length());for(const gp_Pnt& x:p)g3_pnt(x);}
void g3_eval_curve_weights(const NCollection_Array1<double>& w,bool values){g3_int(w.Lower());g3_int(w.Length());if(values)for(double x:w)g3_real(x);}
void g3_eval_curve_state(const occ::handle<Geom_Curve>& c){
 if(auto t=occ::down_cast<GeomEval_TBezierCurve>(c)){g3_int(t->NbPoles());g3_int(t->Order());g3_real(t->Alpha());
  g3_flag(t->IsRational());g3_eval_curve_poles(t->Poles());g3_eval_curve_weights(t->Weights(),true);G3_TRY(g3_pnt(t->StartPoint()));G3_TRY(g3_pnt(t->EndPoint()));}
 if(auto t=occ::down_cast<GeomEval_AHTBezierCurve>(c)){g3_int(t->NbPoles());g3_int(t->AlgDegree());g3_real(t->Alpha());
  g3_real(t->Beta());g3_flag(t->IsRational());g3_eval_curve_poles(t->Poles());g3_eval_curve_weights(t->Weights(),t->IsRational());
  G3_TRY(g3_pnt(t->StartPoint()));G3_TRY(g3_pnt(t->EndPoint()));}
 if(auto s=occ::down_cast<GeomEval_SineWaveCurve>(c)){g3_ax2(s->Position());g3_real(s->Amplitude());g3_real(s->Omega());g3_real(s->Phase());}
 if(auto s=occ::down_cast<GeomEval_CircularHelixCurve>(c)){g3_ax2(s->Position());g3_real(s->Radius());g3_real(s->Pitch());}
}
// g3_eval_curve <curve> params: the class accessors of the curve and of its copy, the copy's
// summary, the high-order derivatives that g3_curve_summary does not reach, the refused
// members (Reverse, Reversed, Transform, Transformed) and the depth-limited dumps.
void g3_eval_curve_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;std::vector<double> u=r.reals();
 g3_name(c);g3_eval_curve_state(c);occ::handle<Geom_Curve> copy=occ::down_cast<Geom_Curve>(c->Copy());
 g3_name(copy);g3_eval_curve_state(copy);g3_curve_summary(copy,u);
 for(double v:u)for(int n=6;n<=9;++n)G3_TRY(g3_vec(c->EvalDN(v,n)));
 gp_Trsf t;t.SetTranslation(gp_Vec(1.,2.,3.));
 G3_TRY(c->Reverse());G3_TRY(c->Reversed());G3_TRY(c->Transform(t));G3_TRY(c->Transformed(t));
 g3_dump(c,0);g3_dump(c,1);g3_dump(c,2);g3_dump(c,-1);std::cout<<'\n';
}
