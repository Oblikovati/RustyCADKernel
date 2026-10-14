#pragma once
// Controlled contents, not replacement geometry or a stencil-state implementation.
struct NativeStencilSeed {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glClear_t original;
  int seed;
  static thread_local NativeStencilSeed* active;
  NativeStencilSeed(const occ::handle<OpenGl_Context>& ctx,int value):context(ctx),original(ctx->core11fwd->glClear),seed(value) {
    active=this;context->core11fwd->glClear=observe;
  }
  ~NativeStencilSeed() {context->core11fwd->glClear=original;active=nullptr;}
  static void APIENTRY observe(GLbitfield mask) {
    const auto gl=active->context->core11fwd;
    const bool frame=(mask&(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT))==(GL_COLOR_BUFFER_BIT|GL_DEPTH_BUFFER_BIT)
      &&(mask&GL_STENCIL_BUFFER_BIT)==0;
    GLint previous=0;
    GLint bits=0,framebuffer=0;gl->glGetIntegerv(GL_STENCIL_BITS,&bits);
    gl->glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING,&framebuffer);
    active->original(mask);
    if(frame&&bits==8) {
      gl->glGetIntegerv(GL_STENCIL_CLEAR_VALUE,&previous);gl->glClearStencil(active->seed);
      active->original(GL_STENCIL_BUFFER_BIT);gl->glClearStencil(previous);
    }
    std::cerr<<"STENCIL_CLEAR={\"mask\":"<<mask<<",\"framebuffer\":"<<framebuffer
      <<",\"bits\":"<<bits<<",\"scene_seed_applied\":"<<(frame&&bits==8?active->seed:-1)<<"}"<<std::endl;
  }
};
thread_local NativeStencilSeed* NativeStencilSeed::active=nullptr;
