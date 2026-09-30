// GeomFill_SectionPlacement probe operation (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_fill_placement.rs. The readers are those of
// tkgeomalgo_probe_geomfill_trihedron.cxx and tkgeomalgo_probe_geomfill_laws.cxx.
#include <GeomFill_SectionPlacement.hxx>
#include <Geom_CartesianPoint.hxx>
#include <memory>
void ga_gfp_state(const GeomFill_SectionPlacement& p){
 ga_flag(p.IsDone());ga_real(p.ParameterOnPath());ga_real(p.ParameterOnSection());ga_real(p.Distance());ga_real(p.Angle());}
void ga_gfp_trsf(const gp_Trsf& t){for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(t.Value(i,j));}
// The state, the four transformations and, for a curve section (a point section has a
// null curve, a crash natively), the placed sections.
void ga_gfp_outputs(const GeomFill_SectionPlacement& p,bool curve){
 ga_gfp_state(p);
 for(int k=0;k<4;++k)GA_TRY(ga_gfp_trsf(p.Transformation((k&1)!=0,(k&2)!=0)));
 if(!curve)return;
 for(int k=0;k<2;++k){GA_TRY(ga_gfl_curve(p.Section(k!=0)));GA_TRY(ga_gfl_curve(p.ModifiedSection(k!=0)));}
}
// `gfs_placement law path point|curve tol param`: a CurveAndTrihedron law on `path`,
// the section (`1 x y z` a Geom_CartesianPoint, `0 curve` a Geom_Curve), then
// Perform(Tol), Perform(Param, Tol) and Perform(Path, Tol), each with its outputs.
void ga_gfp_placement_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_LocationLaw> l;GA_TRY(l=new GeomFill_CurveAndTrihedron(ga_gft_law(r)));
 if(l.IsNull()){ga_end();return;}
 occ::handle<Adaptor3d_Curve> c=ga_gft_curve(r);GA_TRY(ga_flag(l->SetCurve(c)));
 bool point=r.flag();occ::handle<Geom_Geometry> s;
 if(point)s=new Geom_CartesianPoint(r.pnt());else s=ga_ics_geom_curve(r);
 double tol=r.real(),param=r.real();
 std::unique_ptr<GeomFill_SectionPlacement> p;GA_TRY(p=std::make_unique<GeomFill_SectionPlacement>(l,s));
 if(!p){ga_end();return;}
 ga_gfp_state(*p);
 GA_TRY(p->Perform(tol));ga_gfp_outputs(*p,!point);
 GA_TRY(p->Perform(param,tol));ga_gfp_outputs(*p,!point);
 p->SetLocation(l);GA_TRY(p->Perform(c,tol/10));ga_gfp_outputs(*p,!point);
 ga_end();
}
