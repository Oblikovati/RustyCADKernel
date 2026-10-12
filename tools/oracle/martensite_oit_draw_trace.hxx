#pragma once
// Observe actual geometry/composite sample counts; requested parameters alone are insufficient.
struct NativeOitDrawTrace {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawElements_t original;
  OpenGl_GlFunctions::glDrawArrays_t originalArrays;
  std::ofstream output;
  static thread_local NativeOitDrawTrace* active;
  NativeOitDrawTrace(const occ::handle<OpenGl_Context>& ctx,const std::string& prefix)
    :context(ctx),original(ctx->core11fwd->glDrawElements),originalArrays(ctx->core11fwd->glDrawArrays),output(prefix+".draw-trace") {
    active=this;context->core11fwd->glDrawElements=observe;context->core11fwd->glDrawArrays=observeArrays;
  }
  ~NativeOitDrawTrace() {
    context->core11fwd->glDrawElements=original;
    context->core11fwd->glDrawArrays=originalArrays;active=nullptr;
  }
  static void APIENTRY observe(GLenum mode,GLsizei count,GLenum type,const void* indices) {
    active->emit(mode,count);
    active->original(mode,count,type,indices);
  }
  static void APIENTRY observeArrays(GLenum mode,GLint first,GLsizei count) {
    active->emit(mode,count);
    active->originalArrays(mode,first,count);
  }
  int integer(GLenum parameter) {
    GLint value=0;context->core11fwd->glGetIntegerv(parameter,&value);
    return value;
  }
  float scalar(GLenum parameter) {
    GLfloat value=0;context->core11fwd->glGetFloatv(parameter,&value);return value;
  }
  void emit(GLenum mode,GLsizei count) {
    GLboolean mask[4]={};context->core11fwd->glGetBooleanv(GL_COLOR_WRITEMASK,mask);
    output<<"{\"primitive\":"<<mode<<",\"count\":"<<count
      <<",\"samples\":"<<integer(GL_SAMPLES)<<",\"sample_buffers\":"<<integer(GL_SAMPLE_BUFFERS)
      <<",\"draw_buffers\":["<<integer(GL_DRAW_BUFFER0)<<','<<integer(GL_DRAW_BUFFER1)
      <<','<<integer(GL_DRAW_BUFFER2)<<"],\"program\":"<<integer(GL_CURRENT_PROGRAM)
      <<",\"alpha_to_coverage\":"<<int(context->core11fwd->glIsEnabled(GL_SAMPLE_ALPHA_TO_COVERAGE))
      <<",\"alpha_test\":"<<int(context->core11fwd->glIsEnabled(GL_ALPHA_TEST))
      <<",\"alpha_function\":"<<integer(GL_ALPHA_TEST_FUNC)<<",\"alpha_reference\":"<<scalar(GL_ALPHA_TEST_REF)
      <<",\"color_mask\":["<<int(mask[0])<<','<<int(mask[1])<<','<<int(mask[2])<<','<<int(mask[3])<<"]}\n";
  }
};
thread_local NativeOitDrawTrace* NativeOitDrawTrace::active=nullptr;
