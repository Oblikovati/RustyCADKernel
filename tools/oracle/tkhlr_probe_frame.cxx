// The viewing frame of HLRAlgo_Projector(const gp_Ax2&), the Rust twin of
// crates/tkhlr/examples/hl_probe/frame.rs: the transformation from the global frame to
// gp_Ax3(gp_Ax2(P, N, X)) and its inverse, which the projector stores. It proves the
// TKHLR oracle pipeline before the HLR classes are compared.
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Trsf.hxx>
void hl_frame_print(const gp_Trsf& trsf){
 for(int row=1;row<=3;++row)for(int column=1;column<=4;++column)ga_real(trsf.Value(row,column));}
// `P N X`: gp_Trsf::SetTransformation(gp_Ax3(gp_Ax2(P, N, X))), then the 12 coefficients
// of the transformation and of its inverse, or the raised exception type.
void hl_frame_case(std::istream& input){
 GaReader r{input};gp_Pnt location=r.pnt();gp_Dir normal=r.dir();gp_Dir x=r.dir();
 GA_TRY(gp_Ax3 frame(gp_Ax2(location,normal,x));gp_Trsf trsf;trsf.SetTransformation(frame);
  hl_frame_print(trsf);hl_frame_print(trsf.Inverted()))
 ga_end();}
