// Graphic3d_MaterialAspect, the Rust twin of crates/tkservice/examples/sv_probe/material.rs.
#include <Graphic3d_MaterialAspect.hxx>
void sv_color(const Quantity_Color& c){double r,g,b;c.Values(r,g,b,Quantity_TOC_RGB);ga_real(r);ga_real(g);ga_real(b);}
void sv_material_colors(const Graphic3d_MaterialAspect& m){
 sv_color(m.AmbientColor());sv_color(m.DiffuseColor());sv_color(m.SpecularColor());sv_color(m.EmissiveColor());}
void sv_material_pbr(const Graphic3d_PBRMaterial& p){
 sv_vec4f(p.Color());ga_real(p.Metallic());ga_real(p.NormalizedRoughness());ga_real(p.IOR());sv_vec3f(p.Emission());}
void sv_material_bsdf(const Graphic3d_BSDF& b){
 sv_vec4f(b.Kc);sv_vec3f(b.Kd);sv_vec4f(b.Ks);sv_vec3f(b.Kt);sv_vec3f(b.Le);sv_vec4f(b.Absorption);
 sv_vec4f(b.FresnelCoat.Serialize());sv_vec4f(b.FresnelBase.Serialize());}
// `Name`: the predefined material, every member, and its DumpJson text.
void sv_material_case(std::istream& input){
 GaReader r{input};Graphic3d_MaterialAspect m((Graphic3d_NameOfMaterial)r.integer());
 sv_material_colors(m);ga_real(m.Transparency());ga_real(m.RefractionIndex());ga_real(m.Shininess());
 ga_int(m.MaterialType());ga_int(m.Name());ga_text(m.StringName().ToCString());
 sv_material_bsdf(m.BSDF());sv_material_pbr(m.PBRMaterial());
 std::ostringstream dump;m.DumpJson(dump);ga_text(dump.str());
 ga_end();}
// `Rank`: the static MaterialName and MaterialType of a rank.
void sv_material_rank_case(std::istream& input){
 GaReader r{input};int rank=r.integer();
 GA_TRY(ga_text(Graphic3d_MaterialAspect::MaterialName(rank)))
 GA_TRY(ga_int(Graphic3d_MaterialAspect::MaterialType(rank)))
 ga_end();}
// `Name R G B Delta Shine Transparency Index`: SetColor, IncreaseShine, SetShininess,
// SetTransparency, SetRefractionIndex and SetSpecularColor, then the state after each.
void sv_material_edit_case(std::istream& input){
 GaReader r{input};Graphic3d_MaterialAspect m((Graphic3d_NameOfMaterial)r.integer());
 double c[3];for(double& x:c)x=r.real();float delta=(float)r.real(),shine=(float)r.real(),transp=(float)r.real(),index=(float)r.real();
 Quantity_Color color(c[0],c[1],c[2],Quantity_TOC_RGB);
 m.SetColor(color);ga_int(m.Name());sv_material_colors(m);sv_material_pbr(m.PBRMaterial());
 m.IncreaseShine(delta);ga_real(m.Shininess());
 GA_TRY(m.SetShininess(shine))ga_real(m.Shininess());ga_int(m.Name());
 GA_TRY(m.SetTransparency(transp))ga_real(m.Transparency());ga_real(m.Alpha());ga_real(m.PBRMaterial().Alpha());
 GA_TRY(m.SetRefractionIndex(index))ga_real(m.RefractionIndex());
 Graphic3d_MaterialAspect fresh((Graphic3d_NameOfMaterial)m.RequestedName());fresh.SetSpecularColor(color);
 ga_int(fresh.Name());ga_text(fresh.StringName().ToCString());
 for(int k=0;k<4;++k)ga_flag(fresh.ReflectionMode((Graphic3d_TypeOfReflection)k));
 ga_flag(m.IsEqual(fresh));m.Reset();ga_flag(m.IsEqual(Graphic3d_MaterialAspect(m.RequestedName())));
 ga_end();}
