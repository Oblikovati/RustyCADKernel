// GccInt probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/gcc_int.rs.
#include <GccInt_BCirc.hxx>
#include <GccInt_BElips.hxx>
#include <GccInt_BHyper.hxx>
#include <GccInt_BLine.hxx>
#include <GccInt_BParab.hxx>
#include <GccInt_Bisec.hxx>
// Every accessor of a bisector: its geometry or the raised exception type.
void ga_gccint_accessors(const occ::handle<GccInt_Bisec>& b){
 ga_int(b->ArcType());std::cout<<b->DynamicType()->Name()<<' ';
 ga_flag(b->IsKind(STANDARD_TYPE(GccInt_Bisec)));
 GA_TRY(ga_pnt2d(b->Point()));GA_TRY(ga_lin2d(b->Line()));GA_TRY(ga_circ2d(b->Circle()));
 GA_TRY(ga_hypr2d(b->Hyperbola()));GA_TRY(ga_parab2d(b->Parabola()));GA_TRY(ga_elips2d(b->Ellipse()));
}
// `kind x y dx dy a b`: 0 line, 1 circle (radius a), 2 ellipse (radii a b), 3 hyperbola
// (radii a b), 4 parabola (focal a), built on the frame (x y, dx dy), direct sense.
void ga_gccint_bisec_case(std::istream& input){
 GaReader r{input};int kind=r.integer();gp_Pnt2d p=r.pnt2d();gp_Dir2d d=ga_read_dir2d(r);
 double a=r.real(),b=r.real();gp_Ax22d frame(p,d,true);occ::handle<GccInt_Bisec> bisec;
 if(kind==0)bisec=new GccInt_BLine(gp_Lin2d(p,d));
 else if(kind==1)bisec=new GccInt_BCirc(gp_Circ2d(frame,a));
 else if(kind==2)bisec=new GccInt_BElips(gp_Elips2d(frame,a,b));
 else if(kind==3)bisec=new GccInt_BHyper(gp_Hypr2d(frame,a,b));
 else bisec=new GccInt_BParab(gp_Parab2d(frame,a));
 ga_gccint_accessors(bisec);ga_end();
}
