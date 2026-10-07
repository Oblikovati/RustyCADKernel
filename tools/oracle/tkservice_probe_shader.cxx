// Graphic3d_ShaderObject declaration generator, the Rust twin of
// crates/tkservice/examples/sv_probe/shader.rs.
#include <Graphic3d_ShaderObject.hxx>
// Reads `Count Stages...` as variables named `<prefix><index>`.
Graphic3d_ShaderObject::ShaderVariableList sv_shader_variables(GaReader& r,const char* prefix){
 Graphic3d_ShaderObject::ShaderVariableList list;int n=r.integer();
 for(int i=0;i<n;++i){int stages=r.integer();list.Append(Graphic3d_ShaderObject::ShaderVariable(TCollection_AsciiString(prefix)+TCollection_AsciiString(i),stages));}
 return list;}
// `Kind HasIn HasOut NbGeomVerts NbUniforms Stages... NbInOuts Stages...`: the generating
// CreateFromSource over "void main(){}", the rewritten source and the stage source.
void sv_shader_decl_case(std::istream& input){
 GaReader r{input};int kind=r.integer();bool hasIn=r.flag(),hasOut=r.flag();int verts=r.integer();
 Graphic3d_ShaderObject::ShaderVariableList uniforms=sv_shader_variables(r,"vec4 u");
 Graphic3d_ShaderObject::ShaderVariableList inOuts=sv_shader_variables(r,"vec4 v");
 TCollection_AsciiString src("void main(){}");
 occ::handle<Graphic3d_ShaderObject> s=Graphic3d_ShaderObject::CreateFromSource(src,(Graphic3d_TypeOfShaderObject)kind,uniforms,inOuts,
  TCollection_AsciiString(hasIn?"geomIn":""),TCollection_AsciiString(hasOut?"geomOut":""),verts);
 ga_text(src.ToCString());ga_flag(!s.IsNull());if(!s.IsNull())ga_text(s->Source().ToCString());
 ga_end();}
