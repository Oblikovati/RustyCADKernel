// GeomFill_SweepSectionGenerator, GeomFill_AppSweep and GeomFill_Pipe probe operations
// (tkgeomalgo_probe.cxx); the Rust twin is crates/tkgeomalgo/examples/ga_probe/geom_fill_pipe.rs.
#include <GeomFill_AppSweep.hxx>
#include <GeomFill_Line.hxx>
#include <GeomFill_SweepSectionGenerator.hxx>
// Section(P) with and without derivatives on `np` poles preset to -1.
void ga_gfg_section(const GeomFill_SweepSectionGenerator& g,int p,int np){
 NCollection_Array1<gp_Pnt> po(1,np);NCollection_Array1<gp_Vec> dpo(1,np);NCollection_Array1<gp_Pnt2d> p2(1,1);
 NCollection_Array1<gp_Vec2d> dp2(1,1);NCollection_Array1<double> w(1,np),dw(1,np);
 auto reset=[&](){po.Init(gp_Pnt(-1,-1,-1));dpo.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);};
 reset();GA_TRY(g.Section(p,po,p2,w);ga_int(po.Lower());ga_gfs_pnts(po);ga_gfs_reals(w));
 reset();GA_TRY(ga_flag(g.Section(p,po,dpo,p2,dp2,w,dw));ga_gfs_pnts(po);ga_gfs_vecs(dpo);ga_gfs_reals(w);ga_gfs_reals(dw));
 GA_TRY(ga_real(g.Parameter(p)));
}
void ga_gfg_trsf(const GeomFill_SweepSectionGenerator& g,int i){
 GA_TRY(const gp_Trsf& t=g.Transformation(i);for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)ga_real(t.Value(r,c)));}
// The AppSweep approximation of the sections: done, shape, tolerances and surface.
void ga_gfg_app(GeomFill_SweepSectionGenerator& g,bool withParameters){
 occ::handle<GeomFill_Line> line=new GeomFill_Line(g.NbSections());
 GeomFill_AppSweep app(4,8,Precision::Approximation(),Precision::PApproximation(),0,withParameters);
 GA_TRY(app.Perform(line,g,30));ga_flag(app.IsDone());
 if(!app.IsDone())return;
 int s[6];GA_TRY(app.SurfShape(s[0],s[1],s[2],s[3],s[4],s[5]);for(int v:s)ga_int(v));
 GA_TRY(double t3,t2;app.TolReached(t3,t2);ga_real(t3);ga_real(t2));
 GA_TRY(ga_gfw_surface(new Geom_BSplineSurface(app.SurfPoles(),app.SurfWeights(),app.SurfUKnots(),app.SurfVKnots(),
  app.SurfUMults(),app.SurfVMults(),app.UDegree(),app.VDegree())));
}
// `sweepsec kind ... polynomial withparams`: kind 0 rails `path curve1 curve2 radius`
// (ga_gft_curve), 1 `path radius`, 2 `path first`, 3 `path first last` (geometric
// curves); Perform, the shape, knots and multiplicities, sections 0, 1, 2, the middle,
// the last and one past it, the transformations and the AppSweep approximation.
void ga_gfg_sweepsec_case(std::istream& input){
 GaReader r{input};int kind=r.integer();GeomFill_SweepSectionGenerator g;
 if(kind==0){occ::handle<Adaptor3d_Curve> p=ga_gft_curve(r),c1=ga_gft_curve(r),c2=ga_gft_curve(r);double rad=r.real();GA_TRY(g.Init(p,c1,c2,rad));}
 else{occ::handle<Geom_Curve> p=ga_ics_geom_curve(r);
  if(kind==1){double rad=r.real();GA_TRY(g.Init(p,rad));}
  else if(kind==2){occ::handle<Geom_Curve> f=ga_ics_geom_curve(r);GA_TRY(g.Init(p,f));}
  else{occ::handle<Geom_Curve> f=ga_ics_geom_curve(r),l=ga_ics_geom_curve(r);GA_TRY(g.Init(p,f,l));}}
 bool poly=r.flag(),withp=r.flag();
 bool done=false;GA_TRY(g.Perform(poly);done=true);
 if(!done){ga_end();return;}
 int n=g.NbSections();ga_int(n);int np=0,nk=0,deg=0,n2=0;GA_TRY(g.GetShape(np,nk,deg,n2);ga_int(np);ga_int(nk);ga_int(deg);ga_int(n2));
 GA_TRY(NCollection_Array1<double> k(1,nk);k.Init(-1);g.Knots(k);ga_int(k.Lower());ga_gfs_reals(k));
 GA_TRY(NCollection_Array1<int> m(1,nk);m.Init(-1);g.Mults(m);for(int i=m.Lower();i<=m.Upper();++i)ga_int(m(i)));
 for(int p:{0,1,2,n/2,n,n+1})ga_gfg_section(g,p,np);
 for(int i:{0,1,n/2,n-1,n})ga_gfg_trsf(g,i);
 ga_gfg_app(g,withp);
 ga_end();
}
#include <GeomFill_Pipe.hxx>
// A pipe of constructor `kind`: 0 none, 1 `path radius`, 2 `path sect option`, 3 `path2d
// support sect`, 4 `path first last`, 5 `path n sections...`, 6 `path sect dir`, 7 `path c1
// c2 radius` (geometric), 8 `path c1 c2 radius` (ga_gft_curve), 9 `path guide sect byACR
// rotat`; the curves are geometric (ga_ics_geom_curve) unless noted.
std::unique_ptr<GeomFill_Pipe> ga_gfp_pipe(GaReader& r){
 int kind=r.integer();
 if(kind==0)return std::make_unique<GeomFill_Pipe>();
 if(kind==3){occ::handle<Geom2d_Curve> p=ga_curve2d(r);occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);
  occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);return std::make_unique<GeomFill_Pipe>(p,s,c);}
 if(kind==8){occ::handle<Adaptor3d_Curve> p=ga_gft_curve(r),c1=ga_gft_curve(r),c2=ga_gft_curve(r);double rad=r.real();
  return std::make_unique<GeomFill_Pipe>(p,c1,c2,rad);}
 occ::handle<Geom_Curve> p=ga_ics_geom_curve(r);
 if(kind==1){double rad=r.real();return std::make_unique<GeomFill_Pipe>(p,rad);}
 if(kind==2){occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);int o=r.integer();return std::make_unique<GeomFill_Pipe>(p,c,GeomFill_Trihedron(o));}
 if(kind==4){occ::handle<Geom_Curve> f=ga_ics_geom_curve(r),l=ga_ics_geom_curve(r);return std::make_unique<GeomFill_Pipe>(p,f,l);}
 if(kind==5){int n=r.integer();NCollection_Sequence<occ::handle<Geom_Curve>> s;for(int i=0;i<n;++i)s.Append(ga_ics_geom_curve(r));
  return std::make_unique<GeomFill_Pipe>(p,s);}
 if(kind==6){occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);gp_Dir d=r.dir();return std::make_unique<GeomFill_Pipe>(p,c,d);}
 if(kind==7){occ::handle<Geom_Curve> c1=ga_ics_geom_curve(r),c2=ga_ics_geom_curve(r);double rad=r.real();return std::make_unique<GeomFill_Pipe>(p,c1,c2,rad);}
 occ::handle<Adaptor3d_Curve> g=ga_gft_curve(r);occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);bool acr=r.flag(),rot=r.flag();
 return std::make_unique<GeomFill_Pipe>(p,g,c,acr,rot);
}
void ga_gfp_pipe_state(const GeomFill_Pipe& p){ga_flag(p.IsDone());ga_int(int(p.GetStatus()));ga_flag(p.GenerateParticularCase());}
// `pipe ctor... kpart mode tol poly conti degmax nbseg withparams`: the pipe, then
// Perform(Tol, Polynomial, Conti, DegMax, NbMaxSegment) (mode 0) or Perform(WithParameters,
// Polynomial) (mode 1), and the result.
void ga_gfp_pipe_case(std::istream& input){
 GaReader r{input};std::unique_ptr<GeomFill_Pipe> p;
 bool read=false;GA_TRY(p=ga_gfp_pipe(r);read=true);
 if(!read||!p){ga_end();return;}
 bool kpart=r.flag();int mode=r.integer();double tol=r.real();bool poly=r.flag();int conti=r.integer(),degmax=r.integer(),nbseg=r.integer();
 bool withp=r.flag();
 ga_gfp_pipe_state(*p);p->GenerateParticularCase(kpart);
 if(mode==0){GA_TRY(p->Perform(tol,poly,GeomAbs_Shape(conti),degmax,nbseg));}else{GA_TRY(p->Perform(withp,poly));}
 ga_gfp_pipe_state(*p);ga_flag(p->ExchangeUV());ga_real(p->ErrorOnSurf());ga_gfw_surface(p->Surface());
 ga_end();
}
#include <GeomFill.hxx>
// `surface curve1 curve2`: GeomFill::Surface between two geometric curves.
void ga_gfp_surface_case(std::istream& input){
 GaReader r{input};occ::handle<Geom_Curve> c1=ga_ics_geom_curve(r),c2=ga_ics_geom_curve(r);
 GA_TRY(ga_gfw_surface(GeomFill::Surface(c1,c2)));
 ga_end();
}
