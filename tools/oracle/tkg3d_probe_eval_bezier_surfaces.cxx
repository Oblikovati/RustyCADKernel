// TKG3d oracle: the GeomEval Bezier surfaces (tkg3d_probe.cxx). Surface kinds from 30 of
// g3_surface (filled in by the GeomEval Bezier surface port).
occ::handle<Geom_Surface> g3_eval_bezier_surface(int kind,G3Reader& r){
 (void)r;throw Standard_ProgramError(("unknown eval Bezier surface kind "+std::to_string(kind)).c_str());
}
