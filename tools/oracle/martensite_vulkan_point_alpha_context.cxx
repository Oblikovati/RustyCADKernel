// OCCT 3d097a0 context-sequence oracle: preserve prior alpha state and reset through real opaque sprites.
#define main martensite_original_reference_main
#include "martensite_vulkan_reference.cxx"
#undef main
#include "martensite_text_culling_probe.hxx"
#include <OpenGl_ShaderManager.hxx>
#include <OpenGl_ShaderProgram.hxx>
struct NativePointDrawObserver {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawArrays_t original;
  static thread_local NativePointDrawObserver* active;
  NativePointDrawObserver(const occ::handle<OpenGl_Context>& ctx)
      : context(ctx), original(ctx->core11fwd->glDrawArrays) {
    active=this;
    ctx->core11fwd->glDrawArrays=observe;
  }
  ~NativePointDrawObserver() {
    context->core11fwd->glDrawArrays=original;
    active=nullptr;
  }
  static void APIENTRY observe(GLenum mode, GLint first, GLsizei size) {
    if(mode==GL_POINTS) active->emit(size);
    active->original(mode,first,size);
  }
  void emit(GLsizei size) {
    GLint program=0; GLfloat ref=0,color[4]={};
    context->core11fwd->glGetIntegerv(GL_CURRENT_PROGRAM,&program);
    context->core11fwd->glGetFloatv(GL_ALPHA_TEST_REF,&ref);
    GLboolean depthWrite=0; GLint depthFunc=0, fbo=0;
    context->core11fwd->glGetBooleanv(GL_DEPTH_WRITEMASK,&depthWrite);
    context->core11fwd->glGetIntegerv(GL_DEPTH_FUNC,&depthFunc);
    context->core11fwd->glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING,&fbo);
    const GLint location=context->core20fwd->glGetUniformLocation(program,"occColor");
    if(location>=0) context->core20fwd->glGetUniformfv(program,location,color);
    std::cout << "{\"phase\":\"point-draw\",\"count\":"<<size<<",\"clamp\":"<<int(context->core11fwd->glIsEnabled(GL_DEPTH_CLAMP))<<",\"alpha_test\":"<<int(context->core11fwd->glIsEnabled(GL_ALPHA_TEST))<<",\"ref\":"<<ref<<",\"color_alpha\":"<<color[3]<<",\"depth_write\":"<<int(depthWrite)<<",\"depth_test\":"<<int(context->core11fwd->glIsEnabled(GL_DEPTH_TEST))<<",\"depth_func\":"<<depthFunc<<",\"fbo\":"<<fbo<<",\"arb_clamp\":"<<int(context->arbDepthClamp)<<"}\n";
    emit_matrices(program);
  }
  void emit_matrices(GLint program) {
    for(const char* name : {"occProjectionMatrix", "occWorldViewMatrix", "occModelWorldMatrix"}) {
      GLfloat values[16]={}; const GLint matrixLocation=context->core20fwd->glGetUniformLocation(program,name);
      if(matrixLocation>=0) context->core20fwd->glGetUniformfv(program,matrixLocation,values);
      std::cout << "{\"matrix\":\""<<name<<"\",\"values\":[";
      for(int index=0;index<16;++index) std::cout<<(index?",":"")<<values[index];
      std::cout<<"]}\n";
    }
  }
};
thread_local NativePointDrawObserver* NativePointDrawObserver::active=nullptr;
static void print_alpha_state(const NativeTextPolicyProbe& probe, const char* phase) {
  auto context = probe.workspace->GetGlContext();
  context->MakeCurrent();
  GLfloat reference = -1.f;
  GLint function = 0;
  context->core11fwd->glGetFloatv(GL_ALPHA_TEST_REF, &reference);
  context->core11fwd->glGetIntegerv(GL_ALPHA_TEST_FUNC, &function);
  std::cout << "{\"phase\":\"" << phase << "\",\"alpha_test\":"
            << int(context->core11fwd->glIsEnabled(GL_ALPHA_TEST))
            << ",\"reference\":" << reference << ",\"function\":" << function << ",\"material_cutoff\":"
            << context->ShaderManager()->MaterialState().AlphaCutoff() << "}\n";
}
static void capture_opaque_persistent(NativeTextPolicyProbe& probe,
 const occ::handle<Graphic3d_Structure>& structure, const std::string& prefix) {
  structure->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  alpha_point_scene(structure, "point-alpha-opaque");
  structure->SetTransformPersistence(new Graphic3d_TransformPers(Graphic3d_TMF_CameraPers));
  print_alpha_state(probe, "before-opaque-persistence");
  write_capture(probe.view, 64, 64, prefix, false);
  print_alpha_state(probe, "after-opaque-persistence");
}
static void reset_opaque_sprite(NativeTextPolicyProbe& probe,
 const occ::handle<Graphic3d_Structure>& structure, const std::string& prefix) {
  alpha_point_scene(structure, "sprite-alpha-opaque");
  write_capture(probe.view, 64, 64, prefix + "-reset-sprite", false);
  print_alpha_state(probe, "after-opaque-sprite-reset");
  capture_opaque_persistent(probe, structure, prefix + "-reset");
}
static void run_context_sequence(const std::string& prefix) {
  auto probe = create_policy_probe();
  configure_camera(probe.view, 64, 64, 0);
  auto structure = triangle_scene(probe.viewer, 0.);
  const std::string modes[] = {"point-alpha-opaque","point-alpha-auto","point-alpha-blend","point-alpha-mask","point-alpha-forced","point-alpha-zero","point-alpha-cutoff","point-alpha-vertices","point-alpha-depth","point-alpha-clipped","sprite-alpha-aspect","sprite-alpha-opaque","sprite-alpha-auto","sprite-alpha-blend","sprite-alpha-mask","sprite-alpha-user","sprite-alpha-user-vertices","sprite-alpha-zero","sprite-alpha-vertices","sprite-alpha-color","sprite-alpha-color-vertices"};
  for (const auto& mode : modes) {
    structure->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
    alpha_point_scene(structure, mode);
    write_capture(probe.view, 64, 64, prefix + "-" + mode, false);
  }
  print_alpha_state(probe, "after-prior-sequence");
  { NativePointDrawObserver observer(probe.workspace->GetGlContext());
    capture_opaque_persistent(probe, structure, prefix + "-retained");
    reset_opaque_sprite(probe, structure, prefix);
  }
  structure->Erase(); probe.view->Remove();
}
int main(int argc, char** argv) {
  if (argc != 2) {
    std::cerr << "received " << argc << " arguments; expected output-prefix\n";
    return 2;
  }
  run_context_sequence(argv[1]);
  return 0;
}
