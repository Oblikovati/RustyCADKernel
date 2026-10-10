// Test-only control: match RADV's 8-bit viewport snapping without changing
// Martensite. Mesa 26.1.6 si_state_viewport.c selects 12-bit snapping below
// 1024px.
#include <Graphic3d_ShaderObject.hxx>
#include <Graphic3d_ShaderProgram.hxx>
static const char *raster_control_vertex = R"glsl(
uniform vec2 viewportSize;
void main() {
  vec4 clip = occProjectionMatrix * occWorldViewMatrix * occModelWorldMatrix * occVertex;
  vec2 pixel = (clip.xy / clip.w + vec2(1.0)) * viewportSize * 0.5;
  pixel = roundEven(pixel * 256.0) / 256.0;
  clip.xy = (pixel * 2.0 / viewportSize - vec2(1.0)) * clip.w;
  gl_Position = clip;
}
)glsl";
static occ::handle<Graphic3d_ShaderProgram>
raster_control_program(const occ::handle<V3d_View> &view) {
  occ::handle<Graphic3d_ShaderProgram> program = new Graphic3d_ShaderProgram();
  program->SetHeader("#version 150");
  program->AttachShader(Graphic3d_ShaderObject::CreateFromSource(
      Graphic3d_TOS_VERTEX, raster_control_vertex));
  program->AttachShader(Graphic3d_ShaderObject::CreateFromSource(
      Graphic3d_TOS_FRAGMENT,
      "void main() { occSetFragColor(vec4(1.0, 0.0, 0.0, 1.0)); }"));
  int width = 0, height = 0;
  view->Window()->Size(width, height);
  program->PushVariableVec2(
      "viewportSize", NCollection_Vec2<float>(float(width), float(height)));
  return program;
}
static bool raster_control_option(int argc, char **argv) {
  if (argc != 8)
    return false;
  if (std::string(argv[7]) != "raster8" ||
      std::string(argv[6]).rfind("pers-", 0) != 0)
    throw Standard_Failure(
        "received invalid raster control; expected pers-* mode and raster8");
  return true;
}
