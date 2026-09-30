// BRepBndLib::AddOBB probe operation over shapes read by tp_shape (tktopalgo_probe_shape.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/brep_bnd_lib_obb.rs.
#include <BRepBndLib.hxx>
#include <Bnd_OBB.hxx>
// An OBB: void flag, centre, the three axes, the three half sizes and the axis-aligned flag
// (every field, also for a void box).
void tp_obb(const Bnd_OBB& b){
 ga_flag(b.IsVoid());ga_xyz(b.Center());ga_xyz(b.XDirection());ga_xyz(b.YDirection());ga_xyz(b.ZDirection());
 ga_real(b.XHSize());ga_real(b.YHSize());ga_real(b.ZHSize());ga_flag(b.IsAABox());}
// `pre [centre xdir ydir zdir hx hy hz aa]`: a void box (pre 0) or an initial box.
Bnd_OBB tp_initial_obb(GaReader& r){
 Bnd_OBB b;if(!r.flag())return b;
 gp_Pnt c=r.pnt();gp_Dir x=r.dir(),y=r.dir(),z=r.dir();double hx=r.real(),hy=r.real(),hz=r.real();
 b.SetCenter(c);b.SetXComponent(x,hx);b.SetYComponent(y,hy);b.SetZComponent(z,hz);b.SetAABox(r.flag());return b;}
// `shape pre triangulation optimal tolerance`: AddOBB into the box; prints the box, or
// the exception and the box as far as it got.
void tp_bbl_obb_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);Bnd_OBB b=tp_initial_obb(r);bool t=r.flag(),o=r.flag(),g=r.flag();
  GA_TRY(BRepBndLib::AddOBB(s.shape,b,t,o,g))
  tp_obb(b))
 ga_end();}
