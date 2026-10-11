// Observe real native glyph draws without replacing native setupMatrix or drawing algorithms.
#include <OpenGl_ShaderProgram.hxx>
#include <iomanip>
#include <fstream>
struct NativeDecalDrawTrace {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawArrays_t original;
  std::ofstream stream;
  int count = 0;
  int limit;
  // OpenGL exposes a C callback table; per-thread state scopes one injected observer, not renderer globals.
  static thread_local NativeDecalDrawTrace* active;
  NativeDecalDrawTrace(const occ::handle<OpenGl_Context>& ctx, const std::string& prefix, int draws = 5)
      : context(ctx), original(ctx->core11fwd->glDrawArrays), stream(prefix + ".trace"), limit(draws) {
    active = this; context->core11fwd->glDrawArrays = observe;
    stream << std::setprecision(9);
  }
  ~NativeDecalDrawTrace() {
    context->core11fwd->glDrawArrays = original;
    active = nullptr;
  }
  static void APIENTRY observe(GLenum mode, GLint first, GLsizei size) {
    GLint texture = 0; active->context->core11fwd->glGetIntegerv(GL_TEXTURE_BINDING_2D, &texture);
    if (((mode == GL_TRIANGLES && texture != 0) || mode == GL_TRIANGLE_STRIP) && active->count < active->limit) active->emit(size);
    active->original(mode, first, size);
  }
  void emit(int vertices) {
    const auto clip = uniform_matrix("occProjectionMatrix") * uniform_matrix("occWorldViewMatrix") * uniform_matrix("occModelWorldMatrix");
    stream << count++ << ' ' << vertices;
    for (int column = 0; column < 4; ++column)
      for (int row = 0; row < 4; ++row) stream << ' ' << clip.GetValue(row, column);
    emit_color_and_policy(); stream << '\n';
  }
  NCollection_Mat4<float> uniform_matrix(const char* name) {
    GLint program = 0; context->core11fwd->glGetIntegerv(GL_CURRENT_PROGRAM, &program);
    const GLint location = context->core20fwd->glGetUniformLocation(program, name);
    NCollection_Mat4<float> matrix;
    if (location < 0) throw Standard_Failure((std::string("uniform ") + name + "; expected active native glyph matrix").c_str());
    context->core20fwd->glGetUniformfv(program, location, matrix.ChangeData());
    return matrix;
  }
  void emit_color_and_policy() {
    GLfloat color[4] = {}; GLint program = 0;
    context->core11fwd->glGetIntegerv(GL_CURRENT_PROGRAM, &program);
    const GLint location = context->core20fwd->glGetUniformLocation(program, "occColor");
    context->core20fwd->glGetUniformfv(program, location, color);
    for (float value : color) stream << ' ' << value;
    GLfloat units = 0; context->core11fwd->glGetFloatv(GL_POLYGON_OFFSET_UNITS, &units);
    stream << ' ' << int(context->core11fwd->glIsEnabled(GL_DEPTH_TEST))
           << ' ' << int(context->core11fwd->glIsEnabled(GL_POLYGON_OFFSET_FILL)) << ' ' << units;
    GLint operation = 0; context->core11fwd->glGetIntegerv(GL_LOGIC_OP_MODE, &operation);
    stream << ' ' << int(context->core11fwd->glIsEnabled(GL_COLOR_LOGIC_OP)) << ' ' << operation
           << ' ' << int(context->core11fwd->glIsEnabled(GL_BLEND));
  }
};
thread_local NativeDecalDrawTrace* NativeDecalDrawTrace::active = nullptr;
