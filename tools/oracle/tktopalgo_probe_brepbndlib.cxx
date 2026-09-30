// BRepBndLib probe operations over shapes read by tp_shape (tktopalgo_probe_shape.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/brep_bnd_lib.rs.
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
// A box: void flag, then its limits (gap included), gap, whole flag and open flags.
void tp_box(const Bnd_Box& b){
 ga_flag(b.IsVoid());if(b.IsVoid())return;
 double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);
 ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1);ga_real(b.GetGap());ga_flag(b.IsWhole());
 ga_flag(b.IsOpenXmin());ga_flag(b.IsOpenXmax());ga_flag(b.IsOpenYmin());ga_flag(b.IsOpenYmax());
 ga_flag(b.IsOpenZmin());ga_flag(b.IsOpenZmax());}
// `pre [x0 y0 z0 x1 y1 z1 gap]`: an empty box (pre 0) or an initial box to extend.
Bnd_Box tp_initial_box(GaReader& r){
 Bnd_Box b;if(!r.flag())return b;
 double x0=r.real(),y0=r.real(),z0=r.real(),x1=r.real(),y1=r.real(),z1=r.real();
 b.Update(x0,y0,z0,x1,y1,z1);b.SetGap(r.real());return b;}
// `shape pre mode flags`: mode 0 Add(S, B, flag1), 1 AddClose(S, B), 2 AddOptimal(S, B,
// flag1, flag2); prints the box, or the exception and the box as far as it got.
void tp_bbl_add_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);Bnd_Box b=tp_initial_box(r);int mode=r.integer();bool f1=r.flag(),f2=r.flag();
  GA_TRY(if(mode==0)BRepBndLib::Add(s.shape,b,f1);else if(mode==1)BRepBndLib::AddClose(s.shape,b);
   else BRepBndLib::AddOptimal(s.shape,b,f1,f2))
  tp_box(b))
 ga_end();}
