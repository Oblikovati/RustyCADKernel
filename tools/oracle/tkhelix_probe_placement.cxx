// The placement of HelixGeom_BuilderHelix::Perform, the Rust twin of
// crates/tkhelix/examples/hx_probe/placement.rs: the displacement from the standard
// frame to gp_Ax3(gp_Ax2(P, N, X)) that moves every coil into its position. It proves
// the TKHelix oracle pipeline before the HelixGeom classes are compared.
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Trsf.hxx>
// `P N X`: gp_Trsf::SetDisplacement(gp_Ax3(), gp_Ax3(gp_Ax2(P, N, X))), then its 12
// coefficients (rows 1..3, columns 1..4) or the raised exception type.
void hx_placement_case(std::istream& input){
 GaReader r{input};gp_Pnt location=r.pnt();gp_Dir normal=r.dir();gp_Dir x=r.dir();
 GA_TRY(gp_Ax3 target(gp_Ax2(location,normal,x));gp_Trsf trsf;trsf.SetDisplacement(gp_Ax3(),target);
  for(int row=1;row<=3;++row)for(int column=1;column<=4;++column)ga_real(trsf.Value(row,column)))
 ga_end();}
