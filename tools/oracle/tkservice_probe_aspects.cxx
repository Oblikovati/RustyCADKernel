// Graphic3d_Aspects and its primitive classes, the Rust twin of
// crates/tkservice/examples/sv_probe/aspects.rs.
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Graphic3d_AspectMarker3d.hxx>
#include <Graphic3d_AspectText3d.hxx>
// The aspects of a corpus kind: 0 Aspects, 1 FillArea3d, 2 Line3d, 3 Marker3d, 4 Text3d.
occ::handle<Graphic3d_Aspects> sv_aspects_of(int kind){
 switch(kind){case 1:return new Graphic3d_AspectFillArea3d();case 2:return new Graphic3d_AspectLine3d();
  case 3:return new Graphic3d_AspectMarker3d();case 4:return new Graphic3d_AspectText3d();default:return new Graphic3d_Aspects();}}
// `Kind Depth HasFont Cutoff Angle`: the aspects with an optional font, alpha cutoff and text
// angle, dumped as JSON at a depth, and the default line and marker settings.
void sv_aspects_dump_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),depth=r.integer();bool hasFont=r.flag();float cutoff=(float)r.real(),angle=(float)r.real();
 occ::handle<Graphic3d_Aspects> a=sv_aspects_of(kind);
 if(hasFont)a->SetTextFont(new TCollection_HAsciiString("Arial"));
 a->SetAlphaMode(a->AlphaMode(),cutoff);a->SetTextAngle(angle);
 std::ostringstream dump;a->DumpJson(dump,depth);ga_text(dump.str());
 ga_int(a->LineType());ga_int(a->LinePattern());ga_real(a->LineWidth());ga_int(a->MarkerType());ga_real(a->MarkerScale());
 ga_int(a->ShadingModel());ga_int(a->InteriorStyle());ga_flag(a->IsMarkerSprite());ga_flag(a->ToDrawEdges());
 ga_end();}
