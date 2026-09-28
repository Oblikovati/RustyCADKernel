// TKG3d oracle: the GeomEval curves (tkg3d_probe.cxx). Curve kinds of g3_curve from 20:
// (filled in by the GeomEval curve port).
occ::handle<Geom_Curve> g3_eval_curve(int kind,G3Reader& r){
 (void)r;throw Standard_ProgramError(("unknown eval curve kind "+std::to_string(kind)).c_str());
}
