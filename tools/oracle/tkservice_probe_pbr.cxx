// Graphic3d_PBRMaterial and Graphic3d_BSDF, the Rust twin of
// crates/tkservice/examples/sv_probe/pbr.rs.
#include <Graphic3d_BSDF.hxx>
#include <Graphic3d_PBRMaterial.hxx>
#include <Image_PixMap.hxx>
void sv_vec3f(const NCollection_Vec3<float>& v){ga_real(v.x());ga_real(v.y());ga_real(v.z());}
void sv_vec4f(const NCollection_Vec4<float>& v){ga_real(v.x());ga_real(v.y());ga_real(v.z());ga_real(v.w());}
// `SizeX SizeY Samples`: GenerateEnvLUT of an RGF image, every texel.
void sv_pbr_lut_case(std::istream& input){
 GaReader r{input};int sx=r.integer(),sy=r.integer();unsigned int n=(unsigned int)r.integer();
 occ::handle<Image_PixMap> lut=new Image_PixMap();lut->InitZero(Image_Format_RGF,sx,sy);
 GA_TRY(Graphic3d_PBRMaterial::GenerateEnvLUT(lut,n);
  for(int y=0;y<sy;++y)for(int x=0;x<sx;++x){const NCollection_Vec2<float>& v=lut->Value<NCollection_Vec2<float>>(y,x);ga_real(v.x());ga_real(v.y());})
 ga_end();}
// `Probability Roughness`: SpecIBLMapSamplesFactor and the roughness of a normalized one.
void sv_pbr_factor_case(std::istream& input){
 GaReader r{input};float p=(float)r.real(),rough=(float)r.real();
 ga_real(Graphic3d_PBRMaterial::SpecIBLMapSamplesFactor(p,rough));ga_real(Graphic3d_PBRMaterial::Roughness(rough));
 ga_end();}
// `R G B A Metallic Roughness IOR Er Eg Eb`: the PBR material, its metallic-roughness BSDF,
// that BSDF normalized, and the PBR material set back from the BSDF.
void sv_pbr_bsdf_case(std::istream& input){
 GaReader r{input};float c[4];for(float& x:c)x=(float)r.real();float m=(float)r.real(),rough=(float)r.real(),ior=(float)r.real();
 float e[3];for(float& x:e)x=(float)r.real();
 GA_TRY(Graphic3d_PBRMaterial pbr;pbr.SetColor(Quantity_ColorRGBA(c[0],c[1],c[2],c[3]));pbr.SetMetallic(m);pbr.SetRoughness(rough);
  pbr.SetIOR(ior);pbr.SetEmission(NCollection_Vec3<float>(e[0],e[1],e[2]));
  Graphic3d_BSDF b=Graphic3d_BSDF::CreateMetallicRoughness(pbr);
  sv_vec4f(b.Kc);sv_vec3f(b.Kd);sv_vec4f(b.Ks);sv_vec3f(b.Kt);sv_vec3f(b.Le);sv_vec4f(b.Absorption);
  sv_vec4f(b.FresnelCoat.Serialize());sv_vec4f(b.FresnelBase.Serialize());
  Graphic3d_BSDF nb=b;nb.Normalize();sv_vec3f(nb.Kd);sv_vec4f(nb.Ks);sv_vec3f(nb.Kt);
  Graphic3d_PBRMaterial back;back.SetBSDF(b);sv_vec4f(back.Color());ga_real(back.Metallic());ga_real(back.NormalizedRoughness());
  ga_real(back.IOR());sv_vec3f(back.Emission()))
 ga_end();}
// `Nr Ng Nb Kr Kg Kb`: Graphic3d_Fresnel::CreateConductor of per-channel indices.
void sv_fresnel_case(std::istream& input){
 GaReader r{input};float n[3],k[3];for(float& x:n)x=(float)r.real();for(float& x:k)x=(float)r.real();
 sv_vec4f(Graphic3d_Fresnel::CreateConductor(NCollection_Vec3<float>(n[0],n[1],n[2]),NCollection_Vec3<float>(k[0],k[1],k[2])).Serialize());
 sv_vec4f(Graphic3d_Fresnel::CreateConductor(n[0],k[0]).Serialize());
 ga_end();}
