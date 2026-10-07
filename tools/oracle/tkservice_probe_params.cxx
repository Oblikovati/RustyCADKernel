// Graphic3d_RenderingParams, Graphic3d_ZLayerSettings and Graphic3d_PresentationAttributes,
// the Rust twin of crates/tkservice/examples/sv_probe/params.rs.
#include <Graphic3d_PresentationAttributes.hxx>
#include <Graphic3d_RenderingParams.hxx>
#include <Graphic3d_ZLayerSettings.hxx>
template<class T> std::string sv_dump_of(const T& value){std::ostringstream out;value.DumpJson(out);return out.str();}
// Replaces the object of each `"Key": {...}` with `?`: the values natively read uninitialised
// memory (the integer chart size through the double dump macro, the unused union bytes).
std::string sv_masked(std::string text,const std::string& key){
 const std::string head="\""+key+"\": {";
 for(size_t at=text.find(head);at!=std::string::npos;at=text.find(head,at+1)){
  size_t open=at+head.size()-1,end=open;int depth=0;
  do{if(text[end]=='{')++depth;else if(text[end]=='}')--depth;++end;}while(depth>0&&end<text.size());
  text.replace(open,end-open,"?");}
 return text;}
// `Resolution`: the default parameters with a resolution, their ratio and dump.
void sv_params_rendering(GaReader& r){
 Graphic3d_RenderingParams p;p.Resolution=(unsigned int)r.integer();ga_real(p.ResolutionRatio());ga_text(sv_masked(sv_masked(sv_dump_of(p),"ChartSize"),"Params.Params3d"));}
// `X Y Z Distance Size Offset Flags(7)`: a Z layer (Offset 0 none, 1 positive, 2 negative).
void sv_params_layer(GaReader& r){
 Graphic3d_ZLayerSettings s;s.SetName("layer");gp_XYZ o=r.xyz();s.SetOrigin(o);
 s.SetCullingDistance(r.real());s.SetCullingSize(r.real());int offset=r.integer();
 if(offset==1)s.SetDepthOffsetPositive();else if(offset==2)s.SetDepthOffsetNegative();
 s.SetImmediate(r.flag());s.SetRaytracable(r.flag());s.SetEnvironmentTexture(r.flag());s.SetEnableDepthTest(r.flag());
 s.SetEnableDepthWrite(r.flag());s.SetClearDepth(r.flag());s.SetRenderInDepthPrepass(r.flag());
 ga_flag(s.HasCullingDistance());ga_flag(s.HasCullingSize());ga_flag(s.OriginTransformation().IsNull());ga_text(sv_dump_of(s));}
// `Method R G B Transparency Layer Mode Aspect`: presentation attributes.
void sv_params_attributes(GaReader& r){
 occ::handle<Graphic3d_PresentationAttributes> a=new Graphic3d_PresentationAttributes();
 a->SetMethod((Aspect_TypeOfHighlightMethod)r.integer());gp_XYZ c=r.xyz();a->SetColor(Quantity_Color(c.X(),c.Y(),c.Z(),Quantity_TOC_RGB));
 a->SetTransparency((float)r.real());a->SetZLayer(r.integer());a->SetDisplayMode(r.integer());
 if(r.flag())a->SetBasicFillAreaAspect(new Graphic3d_AspectFillArea3d());
 ga_real(a->Transparency());ga_real(a->ColorRGBA().Alpha());ga_text(sv_dump_of(*a));}
// `Kind Args`: 0 rendering parameters, 1 Z layer settings, 2 presentation attributes.
void sv_params_case(std::istream& input){
 GaReader r{input};int kind=r.integer();
 if(kind==0)sv_params_rendering(r);else if(kind==1)sv_params_layer(r);else sv_params_attributes(r);
 ga_end();}
