// TKG3d oracle: the GeomEval elementary surfaces (tkg3d_probe.cxx). Surface kinds 20 to 29
// of g3_surface (filled in by the GeomEval elementary surface port).
occ::handle<Geom_Surface> g3_eval_surface(int kind,G3Reader& r){
 (void)r;throw Standard_ProgramError(("unknown eval surface kind "+std::to_string(kind)).c_str());
}
