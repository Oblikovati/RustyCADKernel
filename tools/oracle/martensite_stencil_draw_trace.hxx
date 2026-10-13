#pragma once
#include <OpenGl_ArbFBO.hxx>
// Record actual draw attachment bits and state before the renderer executes each draw.
struct NativeStencilDrawTrace {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawElements_t original;
  OpenGl_GlFunctions::glDrawArrays_t arrays;
  std::ofstream output;
  static thread_local NativeStencilDrawTrace* active;
  NativeStencilDrawTrace(const occ::handle<OpenGl_Context>& ctx,const std::string& prefix)
    :context(ctx),original(ctx->core11fwd->glDrawElements),arrays(ctx->core11fwd->glDrawArrays),output(prefix+".stencil-trace") {
    active=this;context->core11fwd->glDrawElements=observe;context->core11fwd->glDrawArrays=observe_arrays;
  }
  ~NativeStencilDrawTrace(){context->core11fwd->glDrawElements=original;context->core11fwd->glDrawArrays=arrays;active=nullptr;}
  int integer(GLenum query){GLint value=0;context->core11fwd->glGetIntegerv(query,&value);return value;}
  int stencil_pixel() {
    if(integer(GL_STENCIL_BITS)==0)return -1;
    GLint previous=0;context->core11fwd->glGetIntegerv(GL_READ_FRAMEBUFFER_BINDING,&previous);
    context->arbFBO->glBindFramebuffer(GL_READ_FRAMEBUFFER,integer(GL_DRAW_FRAMEBUFFER_BINDING));
    unsigned char value[4]={};context->core11fwd->glReadPixels(32,32,1,1,GL_STENCIL_INDEX,GL_UNSIGNED_BYTE,value);
    context->arbFBO->glBindFramebuffer(GL_READ_FRAMEBUFFER,previous);return value[0];
  }
  void emit(GLenum mode,GLsizei count) {
    const auto gl=context->core11fwd;
    output<<"{\"mode\":"<<mode<<",\"count\":"<<count<<",\"bits\":"<<integer(GL_STENCIL_BITS)
      <<",\"framebuffer\":"<<integer(GL_DRAW_FRAMEBUFFER_BINDING)<<",\"program\":"<<integer(GL_CURRENT_PROGRAM)
      <<",\"stencil_pixel\":"<<stencil_pixel()<<",\"enabled\":"<<int(gl->glIsEnabled(GL_STENCIL_TEST))<<",\"func\":"<<integer(GL_STENCIL_FUNC)
      <<",\"ref\":"<<integer(GL_STENCIL_REF)<<",\"mask\":"<<integer(GL_STENCIL_VALUE_MASK)<<"}\n";
  }
  static void APIENTRY observe(GLenum mode,GLsizei count,GLenum type,const void* indices) {
    active->emit(mode,count);active->original(mode,count,type,indices);
  }
  static void APIENTRY observe_arrays(GLenum mode,GLint first,GLsizei count) {
    active->emit(mode,count);active->arrays(mode,first,count);
  }
};
thread_local NativeStencilDrawTrace* NativeStencilDrawTrace::active=nullptr;
