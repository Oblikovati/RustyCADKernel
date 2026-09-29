// GccEnt probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/gcc_ent.rs. The 2d readers and conic printers
// below serve the GccInt and GccAna probes too.
#include <GccEnt.hxx>
#include <GccEnt_QualifiedCirc.hxx>
#include <GccEnt_QualifiedLin.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab2d.hxx>
// Sequential 2d readers (the corpus only feeds valid directions and radii).
gp_Dir2d ga_read_dir2d(GaReader& r){double x=r.real();return gp_Dir2d(x,r.real());}
gp_Lin2d ga_read_lin2d(GaReader& r){gp_Pnt2d p=r.pnt2d();return gp_Lin2d(p,ga_read_dir2d(r));}
// `x y dx dy radius`: the circle of X axis (x y, dx dy), direct sense.
gp_Circ2d ga_read_circ2d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=ga_read_dir2d(r);return gp_Circ2d(gp_Ax2d(p,d),r.real());}
void ga_dir2d(const gp_Dir2d& d){ga_xy(d.XY());}
void ga_ax22d(const gp_Ax22d& a){ga_pnt2d(a.Location());ga_dir2d(a.XDirection());ga_dir2d(a.YDirection());}
void ga_lin2d(const gp_Lin2d& l){ga_pnt2d(l.Location());ga_dir2d(l.Direction());}
void ga_circ2d(const gp_Circ2d& c){ga_ax22d(c.Axis());ga_real(c.Radius());}
void ga_elips2d(const gp_Elips2d& e){ga_ax22d(e.Axis());ga_real(e.MajorRadius());ga_real(e.MinorRadius());}
void ga_hypr2d(const gp_Hypr2d& h){ga_ax22d(h.Axis());ga_real(h.MajorRadius());ga_real(h.MinorRadius());}
void ga_parab2d(const gp_Parab2d& p){ga_ax22d(p.Axis());ga_real(p.Focal());}
// The position names fed to GccEnt::PositionFromString by index.
static const char* const ga_gccent_names[]={"unqualified","Enclosing","ENCLOSED","outSide","NoQualifier","","bogus","OUTSIDE "};
// `position name_index`: PositionToString, Print, then both PositionFromString overloads.
void ga_gccent_position_case(std::istream& input){
 GaReader r{input};int p=r.integer();const char* name=ga_gccent_names[r.integer()];
 std::cout<<GccEnt::PositionToString(GccEnt_Position(p))<<' ';
 std::ostringstream printed;GccEnt::Print(GccEnt_Position(p),printed);std::cout<<printed.str()<<' ';
 GccEnt_Position found=GccEnt_noqualifier;ga_flag(GccEnt::PositionFromString(name,found));ga_int(found);
 ga_int(GccEnt::PositionFromString(name));ga_end();
}
void ga_gccent_circ_flags(const GccEnt_QualifiedCirc& q){
 ga_int(q.Qualifier());ga_flag(q.IsUnqualified());ga_flag(q.IsEnclosing());ga_flag(q.IsEnclosed());ga_flag(q.IsOutside());
}
void ga_gccent_lin_flags(const GccEnt_QualifiedLin& q){
 ga_int(q.Qualifier());ga_flag(q.IsUnqualified());ga_flag(q.IsEnclosed());ga_flag(q.IsOutside());
}
// `position circle(5) line(4)`: the qualified circle and line, then the seven GccEnt makers.
void ga_gccent_qualified_case(std::istream& input){
 GaReader r{input};GccEnt_Position p=GccEnt_Position(r.integer());
 gp_Circ2d c=ga_read_circ2d(r);gp_Lin2d l=ga_read_lin2d(r);
 GccEnt_QualifiedCirc qc(c,p);ga_circ2d(qc.Qualified());ga_gccent_circ_flags(qc);
 GccEnt_QualifiedLin ql(l,p);ga_lin2d(ql.Qualified());ga_gccent_lin_flags(ql);
 ga_gccent_lin_flags(GccEnt::Unqualified(l));ga_gccent_circ_flags(GccEnt::Unqualified(c));
 ga_gccent_circ_flags(GccEnt::Enclosing(c));ga_gccent_lin_flags(GccEnt::Enclosed(l));
 ga_gccent_circ_flags(GccEnt::Enclosed(c));ga_gccent_lin_flags(GccEnt::Outside(l));
 ga_gccent_circ_flags(GccEnt::Outside(c));ga_end();
}
