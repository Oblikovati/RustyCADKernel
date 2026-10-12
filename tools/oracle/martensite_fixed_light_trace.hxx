#pragma once
// OCCT3d097a0: observe fixed lighting immediately before actual indexed draws.
#include <iomanip>
#include <GL/gl.h>
struct NativeFixedLightingTrace {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawElements_t original;
  OpenGl_GlFunctions::glDrawArrays_t originalArrays;
  std::ofstream output;
  static thread_local NativeFixedLightingTrace* active;
  NativeFixedLightingTrace(const occ::handle<OpenGl_Context>& ctx,const std::string& prefix)
    :context(ctx),original(ctx->core11fwd->glDrawElements),originalArrays(ctx->core11fwd->glDrawArrays),output(prefix+".lighting-trace") {
    active=this;context->core11fwd->glDrawElements=observe;context->core11fwd->glDrawArrays=observeArrays;
    output<<std::setprecision(9);
  }
  ~NativeFixedLightingTrace() {
    context->core11fwd->glDrawElements=original;context->core11fwd->glDrawArrays=originalArrays;active=nullptr;
  }
  static void APIENTRY observe(GLenum mode,GLsizei count,GLenum type,const void* indices) {
    if(count>=3&&count<=6)active->emit(mode);
    active->original(mode,count,type,indices);
  }
  static void APIENTRY observeArrays(GLenum mode,GLint first,GLsizei count) {
    if(count>=3&&count<=12)active->emit(mode);
    active->originalArrays(mode,first,count);
  }
  void floats(GLenum parameter,int count) {
    GLfloat values[16]={};context->core11fwd->glGetFloatv(parameter,values);
    for(int index=0;index<count;++index)output<<','<<values[index];
  }
  void material(GLenum face,GLenum parameter,int count) {
    GLfloat values[4]={};::glGetMaterialfv(face,parameter,values);
    for(int index=0;index<count;++index)output<<','<<values[index];
  }
  void light(GLenum index,GLenum parameter,int count) {
    GLfloat values[4]={};::glGetLightfv(index,parameter,values);
    for(int component=0;component<count;++component)output<<','<<values[component];
  }
  void emit(GLenum mode) {
    const auto gl=context->core11fwd;
    output<<'['<<mode<<','<<int(gl->glIsEnabled(GL_LIGHTING))<<','<<int(gl->glIsEnabled(GL_NORMALIZE))
      <<','<<int(gl->glIsEnabled(GL_COLOR_MATERIAL))<<','<<int(gl->glIsEnabled(GL_NORMAL_ARRAY));
    floats(GL_SHADE_MODEL,1);floats(GL_LIGHT_MODEL_TWO_SIDE,1);floats(GL_LIGHT_MODEL_LOCAL_VIEWER,1);
    floats(GL_LIGHT_MODEL_AMBIENT,4);floats(GL_CURRENT_NORMAL,3);floats(GL_MODELVIEW_MATRIX,16);
    emit_materials();emit_lights();emit_texture_environment();output<<"]\n";
  }
  void emit_texture_environment() {
    GLint mode=0,origin=0;
    ::glGetTexEnviv(GL_TEXTURE_ENV,GL_TEXTURE_ENV_MODE,&mode);
    context->core11fwd->glGetIntegerv(GL_POINT_SPRITE_COORD_ORIGIN,&origin);
    output<<','<<mode<<','<<origin;
    floats(GL_CURRENT_COLOR,4);
  }
  void emit_materials() {
    for(GLenum face:{GL_FRONT,GL_BACK}) {
      for(GLenum parameter:{GL_AMBIENT,GL_DIFFUSE,GL_SPECULAR,GL_EMISSION})material(face,parameter,4);
      material(face,GL_SHININESS,1);
    }
  }
  void emit_lights() {
    for(int index=0;index<8;++index) {
      const GLenum id=GL_LIGHT0+index;output<<','<<int(context->core11fwd->glIsEnabled(id));
      for(GLenum parameter:{GL_POSITION,GL_DIFFUSE,GL_SPECULAR})light(id,parameter,4);
      light(id,GL_SPOT_DIRECTION,3);
      for(GLenum parameter:{GL_SPOT_EXPONENT,GL_SPOT_CUTOFF,GL_CONSTANT_ATTENUATION,GL_LINEAR_ATTENUATION})light(id,parameter,1);
    }
  }
};
thread_local NativeFixedLightingTrace* NativeFixedLightingTrace::active=nullptr;
