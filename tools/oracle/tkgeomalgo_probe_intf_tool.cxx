// Intf_Tool probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/intf_tool.rs.
#include <Bnd_Box.hxx>
#include <Bnd_Box2d.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <Intf_Tool.hxx>
// A 2d domain: `0` void, `1` whole, `2 xmin ymin xmax ymax gap openmask` (bits xmin,
// xmax, ymin, ymax).
Bnd_Box2d ga_box2d(GaReader& r){
 Bnd_Box2d box;int mode=r.integer();
 if(mode==1)box.SetWhole();
 if(mode!=2)return box;
 double x0=r.real(),y0=r.real(),x1=r.real(),y1=r.real();box.Update(x0,y0,x1,y1);box.SetGap(r.real());int open=r.integer();
 if(open&1)box.OpenXmin();if(open&2)box.OpenXmax();if(open&4)box.OpenYmin();if(open&8)box.OpenYmax();
 return box;
}
// A 3d domain: `0` void, `1` whole, `2 xmin ymin zmin xmax ymax zmax gap openmask`.
Bnd_Box ga_box3d(GaReader& r){
 Bnd_Box box;int mode=r.integer();
 if(mode==1)box.SetWhole();
 if(mode!=2)return box;
 double x0=r.real(),y0=r.real(),z0=r.real(),x1=r.real(),y1=r.real(),z1=r.real();box.Update(x0,y0,z0,x1,y1,z1);box.SetGap(r.real());
 int open=r.integer();
 if(open&1)box.OpenXmin();if(open&2)box.OpenXmax();if(open&4)box.OpenYmin();if(open&8)box.OpenYmax();
 if(open&16)box.OpenZmin();if(open&32)box.OpenZmax();
 return box;
}
void ga_box2d_print(const Bnd_Box2d& box){
 ga_flag(box.IsVoid());ga_flag(box.IsWhole());ga_flag(box.IsOpenXmin());ga_flag(box.IsOpenXmax());
 ga_flag(box.IsOpenYmin());ga_flag(box.IsOpenYmax());ga_real(box.GetGap());
 if(box.IsVoid())return;double a,b,c,d;box.Get(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);
}
void ga_box3d_print(const Bnd_Box& box){
 ga_flag(box.IsVoid());ga_flag(box.IsWhole());ga_flag(box.IsOpenXmin());ga_flag(box.IsOpenXmax());ga_flag(box.IsOpenYmin());
 ga_flag(box.IsOpenYmax());ga_flag(box.IsOpenZmin());ga_flag(box.IsOpenZmax());ga_real(box.GetGap());
 if(box.IsVoid())return;double a,b,c,d,e,f;box.Get(a,b,c,d,e,f);ga_real(a);ga_real(b);ga_real(c);ga_real(d);ga_real(e);ga_real(f);
}
// The segments of the last computation, with the range errors on both sides.
void ga_tool_segments_print(const Intf_Tool& tool){
 ga_int(tool.NbSegments());
 for(int i=1;i<=tool.NbSegments();++i){ga_real(tool.BeginParam(i));ga_real(tool.EndParam(i));}
 GA_TRY(ga_real(tool.BeginParam(0)));GA_TRY(ga_real(tool.EndParam(tool.NbSegments()+1)));
}
// Calls on one tool: `0 px py dx dy` (Lin2dBox), `1 px py dx dy sense major minor`
// (Hypr2dBox), `2 px py dx dy sense focal` (Parab2dBox), each followed by a domain.
void ga_intf_tool2d_case(std::istream& input){
 GaReader r{input};Intf_Tool tool;int count=r.integer();
 for(int k=0;k<count;++k){
  int kind=r.integer();gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();Bnd_Box2d result;
  if(kind==0){Bnd_Box2d domain=ga_box2d(r);tool.Lin2dBox(gp_Lin2d(p,d),domain,result);}
  else if(kind==1){bool sense=r.flag();double major=r.real(),minor=r.real();Bnd_Box2d domain=ga_box2d(r);
   GA_TRY(tool.Hypr2dBox(gp_Hypr2d(gp_Ax2d(p,d),major,minor,sense),domain,result));}
  else{bool sense=r.flag();double focal=r.real();Bnd_Box2d domain=ga_box2d(r);
   GA_TRY(tool.Parab2dBox(gp_Parab2d(gp_Ax2d(p,d),focal,sense),domain,result));}
  ga_tool_segments_print(tool);ga_box2d_print(result);
 }
 ga_end();
}
// Calls on one tool: `0 p d` (LinBox), `1 p n x major minor` (HyprBox), `2 p n x focal`
// (ParabBox), each followed by a domain.
void ga_intf_tool3d_case(std::istream& input){
 GaReader r{input};Intf_Tool tool;int count=r.integer();
 for(int k=0;k<count;++k){
  int kind=r.integer();gp_Pnt p=r.pnt();Bnd_Box result;
  if(kind==0){gp_Dir d=r.dir();Bnd_Box domain=ga_box3d(r);tool.LinBox(gp_Lin(p,d),domain,result);}
  else if(kind==1){gp_Dir n=r.dir(),x=r.dir();double major=r.real(),minor=r.real();Bnd_Box domain=ga_box3d(r);
   GA_TRY(tool.HyprBox(gp_Hypr(gp_Ax2(p,n,x),major,minor),domain,result));}
  else{gp_Dir n=r.dir(),x=r.dir();double focal=r.real();Bnd_Box domain=ga_box3d(r);
   GA_TRY(tool.ParabBox(gp_Parab(gp_Ax2(p,n,x),focal),domain,result));}
  ga_tool_segments_print(tool);ga_box3d_print(result);
 }
 ga_end();
}
