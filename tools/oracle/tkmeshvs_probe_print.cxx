// Printers of the TKMeshVS probe, the twin of crates/tkmeshvs/examples/mv_probe/print.rs:
// the recording driver events, texts, and the full final aspects of every group.
static void mv_color(const Quantity_Color& c){ga_real(c.Red());ga_real(c.Green());ga_real(c.Blue());}
static void mv_material(const Graphic3d_MaterialAspect& m){
  ga_int(int(m.Name()));ga_real(m.Transparency());mv_color(m.AmbientColor());mv_color(m.DiffuseColor());
  mv_color(m.SpecularColor());mv_color(m.EmissiveColor());}
static void mv_aspect(const occ::handle<Graphic3d_Aspects>& a){
  ga_flag(a.IsNull());if(a.IsNull())return;
  mv_color(a->InteriorColor());mv_color(a->BackInteriorColor());mv_color(a->EdgeColor());
  ga_int(int(a->InteriorStyle()));ga_flag(a->Distinguish());ga_flag(a->ToDrawEdges());ga_int(int(a->ShadingModel()));
  mv_material(a->FrontMaterial());ga_int(int(a->BackMaterial().Name()));
  ga_int(int(a->LineType()));ga_real(a->LineWidth());ga_int(int(a->MarkerType()));ga_real(a->MarkerScale());
  ga_flag(a->ToMapTexture());ga_flag(a->TextureMap().IsNull());
  ga_int(int(a->TextFontAspect()));ga_int(int(a->TextStyle()));ga_int(int(a->TextDisplayType()));
  ga_text(a->TextFont().IsNull()?std::string():std::string(a->TextFont()->ToCString()));}
static void mv_structure(const occ::handle<Graphic3d_Structure>& prs){
  ga_flag(prs.IsNull());if(prs.IsNull())return;
  ga_int(prs->Groups().Size());for(const auto& g:prs->Groups()){ga_flag(g->IsClosed());mv_aspect(g->Aspects());}}
static void mv_trace(const V3DisplayTrace& trace){
  ga_int(int(trace.events.size()));for(double v:trace.events)ga_real(v);
  ga_int(int(trace.texts.size()));
  for(const auto& t:trace.texts){ga_pnt(t->Position());ga_real(t->Height());ga_text(t->Text().ToCString());}}
