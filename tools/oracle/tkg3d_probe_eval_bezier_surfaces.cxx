// TKG3d oracle: the GeomEval Bezier surfaces (tkg3d_probe.cxx). Surface kinds of g3_surface:
// 30 tbezier rational lr lc nu nv (x y z)*(nu*nv) nwu nwv w*(nwu*nwv) alphaU alphaV |
// 31 aht rational lr lc degU degV alphaU alphaV betaU betaV nu nv (x y z)*(nu*nv) nwu nwv w*(nwu*nwv).
// Poles and weights are read row by row (U major); both grids start at (lr, lc).
#include <GeomEval_AHTBezierSurface.hxx>
#include <GeomEval_TBezierSurface.hxx>
struct G3EvalGrids{NCollection_Array2<gp_Pnt> p;NCollection_Array2<double> w;};
G3EvalGrids g3_eval_grids(G3Reader& r,int lr,int lc){
 int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> p(lr,lr+nu-1,lc,lc+nv-1);
 for(int i=lr;i<lr+nu;++i)for(int j=lc;j<lc+nv;++j)p(i,j)=r.pnt();
 int nwu=r.integer(),nwv=r.integer();NCollection_Array2<double> w(lr,lr+nwu-1,lc,lc+nwv-1);
 for(int i=lr;i<lr+nwu;++i)for(int j=lc;j<lc+nwv;++j)w(i,j)=r.real();
 return G3EvalGrids{p,w};
}
occ::handle<Geom_Surface> g3_eval_bezier_surface(int kind,G3Reader& r){
 bool rational=r.flag();int lr=r.integer(),lc=r.integer();
 if(kind==30){G3EvalGrids g=g3_eval_grids(r,lr,lc);double au=r.real(),av=r.real();
  if(rational)return new GeomEval_TBezierSurface(g.p,g.w,au,av);return new GeomEval_TBezierSurface(g.p,au,av);}
 if(kind!=31)throw Standard_ProgramError(("unknown eval Bezier surface kind "+std::to_string(kind)).c_str());
 int du=r.integer(),dv=r.integer();double au=r.real(),av=r.real(),bu=r.real(),bv=r.real();
 G3EvalGrids g=g3_eval_grids(r,lr,lc);
 if(rational)return new GeomEval_AHTBezierSurface(g.p,g.w,du,dv,au,av,bu,bv);
 return new GeomEval_AHTBezierSurface(g.p,du,dv,au,av,bu,bv);
}
// A grid with its bounds; the values only when stored (the non-rational AHT surface keeps
// one uninitialized weight).
template<class T,class F> void g3_eval_grid(const NCollection_Array2<T>& a,bool values,F print){
 g3_int(a.LowerRow());g3_int(a.LowerCol());g3_int(a.NbRows());g3_int(a.NbColumns());if(values)for(const T& x:a)print(x);
}
void g3_eval_bezier_state(const occ::handle<Geom_Surface>& s){
 if(auto t=occ::down_cast<GeomEval_TBezierSurface>(s)){g3_int(t->NbUPoles());g3_int(t->NbVPoles());g3_int(t->OrderU());
  g3_int(t->OrderV());g3_real(t->AlphaU());g3_real(t->AlphaV());g3_flag(t->IsRational());
  g3_eval_grid(t->Poles(),true,g3_pnt);g3_eval_grid(t->Weights(),true,g3_real);}
 if(auto a=occ::down_cast<GeomEval_AHTBezierSurface>(s)){g3_int(a->NbPolesU());g3_int(a->NbPolesV());
  g3_int(a->AlgDegreeU());g3_int(a->AlgDegreeV());g3_real(a->AlphaU());g3_real(a->AlphaV());g3_real(a->BetaU());
  g3_real(a->BetaV());g3_flag(a->IsURational());g3_flag(a->IsVRational());
  g3_eval_grid(a->Poles(),true,g3_pnt);g3_eval_grid(a->Weights(),a->IsURational(),g3_real);}
}
// g3_eval_bezier_surface <surface> uv-list trsf: the class accessors of the surface and of its
// copy, the copy's summary, EvalDN above the summary's orders, the refused members and dumps.
void g3_eval_bezier_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 std::vector<double> uv=r.reals();gp_Trsf t=g3_trsf(r);
 g3_name(s);g3_eval_bezier_state(s);occ::handle<Geom_Surface> copy=occ::down_cast<Geom_Surface>(s->Copy());
 g3_name(copy);g3_eval_bezier_state(copy);g3_surface_summary(copy,uv);
 for(size_t i=0;i+1<uv.size();i+=2)for(int nu=0;nu<=5;++nu)for(int nv=4-nu<0?0:4-nu;nv<=6-nu;++nv)
  G3_TRY(g3_vec(s->EvalDN(uv[i],uv[i+1],nu,nv)));
 G3_TRY(g3_vec(s->EvalDN(0.5,0.5,2,-1)));G3_TRY(g3_vec(s->EvalDN(0.5,0.5,0,0)));
 G3_TRY(s->UReverse());G3_TRY(s->VReverse());G3_TRY(g3_name(s->UIso(0.5)));G3_TRY(g3_name(s->VIso(0.5)));
 G3_TRY(s->Transform(t));G3_TRY(g3_name(s->Transformed(t)));G3_TRY(g3_name(s->UReversed()));
 g3_dump(s,-1);g3_dump(s,0);g3_dump(s,1);g3_dump(s,2);std::cout<<'\n';
}
