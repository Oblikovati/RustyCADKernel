// TKG3d oracle: the GeomEval elementary surfaces (tkg3d_probe.cxx). Surface kinds 20 to 29
// of g3_surface, where F is a gp_Ax3 (P N X):
// 20 ellipsoid F A B C | 21 hyperboloid F R1 R2 mode | 22 paraboloid F focal |
// 23 hyperbolic paraboloid F A B | 24 circular helicoid F pitch.
#include <GeomEval_CircularHelicoidSurface.hxx>
#include <GeomEval_EllipsoidSurface.hxx>
#include <GeomEval_HypParaboloidSurface.hxx>
#include <GeomEval_HyperboloidSurface.hxx>
#include <GeomEval_ParaboloidSurface.hxx>
occ::handle<Geom_Surface> g3_eval_surface(int kind,G3Reader& r){
 gp_Ax3 f=r.ax3();double a=r.real();
 if(kind==22)return new GeomEval_ParaboloidSurface(f,a);
 if(kind==24)return new GeomEval_CircularHelicoidSurface(f,a);
 double b=r.real();
 if(kind==23)return new GeomEval_HypParaboloidSurface(f,a,b);
 double c=r.real();
 if(kind==20)return new GeomEval_EllipsoidSurface(f,a,b,c);
 if(kind==21)return new GeomEval_HyperboloidSurface(f,a,b,GeomEval_HyperboloidSurface::SheetMode(int(c)));
 throw Standard_ProgramError(("unknown eval surface kind "+std::to_string(kind)).c_str());
}
// The implicit equation of the quadrics (the helicoid has none).
template<class T> void g3_eval_coefficients(const T& s){
 double c[10];s->Coefficients(c[0],c[1],c[2],c[3],c[4],c[5],c[6],c[7],c[8],c[9]);for(double v:c)g3_real(v);
}
// The class getters and Coefficients.
void g3_eval_surface_state(const occ::handle<Geom_Surface>& s){
 if(auto e=occ::down_cast<GeomEval_EllipsoidSurface>(s)){g3_real(e->SemiAxisA());g3_real(e->SemiAxisB());g3_real(e->SemiAxisC());g3_eval_coefficients(e);}
 if(auto h=occ::down_cast<GeomEval_HyperboloidSurface>(s)){g3_real(h->R1());g3_real(h->R2());g3_int(int(h->Mode()));g3_eval_coefficients(h);}
 if(auto p=occ::down_cast<GeomEval_ParaboloidSurface>(s)){g3_real(p->Focal());g3_eval_coefficients(p);}
 if(auto h=occ::down_cast<GeomEval_HypParaboloidSurface>(s)){g3_real(h->SemiAxisA());g3_real(h->SemiAxisB());g3_eval_coefficients(h);}
 if(auto h=occ::down_cast<GeomEval_CircularHelicoidSurface>(s))g3_real(h->Pitch());
}
// The class setters with `a`, `b` and `c` (each may raise and leave the state unchanged).
void g3_eval_surface_set(const occ::handle<Geom_Surface>& s,double a,double b,double c){
 if(auto e=occ::down_cast<GeomEval_EllipsoidSurface>(s)){G3_TRY(e->SetSemiAxisA(a));G3_TRY(e->SetSemiAxisB(b));G3_TRY(e->SetSemiAxisC(c));}
 if(auto h=occ::down_cast<GeomEval_HyperboloidSurface>(s)){G3_TRY(h->SetR1(a));G3_TRY(h->SetR2(b));
  h->SetMode(GeomEval_HyperboloidSurface::SheetMode(int(c)));}
 if(auto p=occ::down_cast<GeomEval_ParaboloidSurface>(s))G3_TRY(p->SetFocal(a));
 if(auto h=occ::down_cast<GeomEval_HypParaboloidSurface>(s)){G3_TRY(h->SetSemiAxisA(a));G3_TRY(h->SetSemiAxisB(b));}
 if(auto h=occ::down_cast<GeomEval_CircularHelicoidSurface>(s))G3_TRY(h->SetPitch(a));
}
// Class members: <surface> uv-list a b c. The state and summary of the surface and of its
// copy, EvalDN up to order 6, then the setters, the new state and summary, and the dumps.
void g3_eval_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 std::vector<double> uv=r.reals();double a=r.real(),b=r.real(),c=r.real();
 g3_name(s);g3_eval_surface_state(s);g3_surface_summary(s,uv);
 occ::handle<Geom_Surface> copy=occ::down_cast<Geom_Surface>(s->Copy());g3_name(copy);g3_eval_surface_state(copy);
 for(size_t i=0;i+1<uv.size();i+=2)for(int nu=0;nu<=6;++nu)for(int nv=4-nu>0?4-nu:0;nv<=6-nu;++nv)
  G3_TRY(g3_vec(s->EvalDN(uv[i],uv[i+1],nu,nv)));
 g3_eval_surface_set(s,a,b,c);g3_eval_surface_state(s);g3_surface_summary(s,uv);
 g3_eval_surface_state(copy);g3_dump(s,-1);g3_dump(s,0);g3_dump(copy,1);std::cout<<'\n';
}
