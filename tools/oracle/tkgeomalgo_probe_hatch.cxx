// Hatch probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/hatch.rs.
#include <Hatch_Hatcher.hxx>
#include <Hatch_Line.hxx>
#include <Hatch_Parameter.hxx>
// One hatching line: 0 `x y dx dy form` (AddLine), 1 `dx dy dist`, 2 `x` (AddXLine), 3 `y`.
void ga_hatch_add_line(GaReader& r,Hatch_Hatcher& h){
 int kind=r.integer();
 if(kind==0){gp_Lin2d l=ga_read_lin2d(r);h.AddLine(l,Hatch_LineForm(r.integer()));}
 else if(kind==1){gp_Dir2d d=r.dir2d();h.AddLine(d,r.real());}
 else if(kind==2)h.AddXLine(r.real());
 else h.AddYLine(r.real());
}
// One trimming: 0 `x y dx dy index` (whole line), 1 `x y dx dy start end index`, 2 `p1 p2 index`.
void ga_hatch_trim(GaReader& r,Hatch_Hatcher& h){
 int kind=r.integer();
 if(kind==0){gp_Lin2d l=ga_read_lin2d(r);int index=r.integer();GA_TRY(h.Trim(l,index));}
 else if(kind==1){gp_Lin2d l=ga_read_lin2d(r);double s=r.real(),e=r.real();int index=r.integer();GA_TRY(h.Trim(l,s,e,index));}
 else{gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d();int index=r.integer();GA_TRY(h.Trim(p1,p2,index));}
}
// The intervals of line `i`, queried from J = 0 to NbIntervals + 1.
void ga_hatch_line_report(const Hatch_Hatcher& h,int i){
 GA_TRY(ga_lin2d(h.Line(i)));GA_TRY(ga_int(h.LineForm(i)));GA_TRY(ga_flag(h.IsXLine(i)));
 GA_TRY(ga_flag(h.IsYLine(i)));GA_TRY(ga_real(h.Coordinate(i)));
 int n=-1;GA_TRY(n=h.NbIntervals(i);ga_int(n));
 for(int j=0;j<=n+1;j++){
  int index=-7;double par2=-7.;
  GA_TRY(ga_real(h.Start(i,j)));GA_TRY(h.StartIndex(i,j,index,par2);ga_int(index);ga_real(par2));
  GA_TRY(ga_real(h.End(i,j)));GA_TRY(h.EndIndex(i,j,index,par2);ga_int(index);ga_real(par2));
 }
}
// `tol oriented nlines lines... ntrims trims...`: the hatcher state after the trimmings,
// line by line from index 0 to NbLines + 1.
void ga_hatch_hatcher_case(std::istream& input){
 GaReader r{input};double tol=r.real();Hatch_Hatcher h(tol,r.flag());
 int nlines=r.integer();for(int k=0;k<nlines;k++)ga_hatch_add_line(r,h);
 int ntrims=r.integer();for(int k=0;k<ntrims;k++)ga_hatch_trim(r,h);
 ga_real(h.Tolerance());h.Tolerance(tol*2.);ga_real(h.Tolerance());h.Tolerance(tol);
 ga_int(h.NbLines());ga_int(h.NbIntervals());
 for(int i=0;i<=h.NbLines()+1;i++)ga_hatch_line_report(h,i);
 ga_end();
}
