#pragma once
// Observe actual homogeneous cap draws; OCCT3d097a0, without changing native raster state.
#include <fstream>
#include <iomanip>
#include <GL/gl.h>
struct NativeCapMeshDrawTrace {
  occ::handle<OpenGl_Context> context;
  OpenGl_GlFunctions::glDrawArrays_t original;
  std::ofstream stream;
  static thread_local NativeCapMeshDrawTrace* active;
  NativeCapMeshDrawTrace(const occ::handle<OpenGl_Context>& ctx,const std::string& prefix)
    :context(ctx),original(ctx->core11fwd->glDrawArrays),stream(prefix+".mesh-trace") {
    active=this;context->core11fwd->glDrawArrays=observe;stream<<std::setprecision(9);
  }
  ~NativeCapMeshDrawTrace() {context->core11fwd->glDrawArrays=original;active=nullptr;}
  static void APIENTRY observe(GLenum mode,GLint first,GLsizei count) {
    if(mode==GL_TRIANGLES&&count==12)active->emit();
    active->original(mode,first,count);
  }
  void emit() {
    GLint polygon[2]={},program=0,texture=0,depth=0;
    GLfloat factor=0,units=0;
    const auto gl=context->core11fwd;
    gl->glGetIntegerv(GL_POLYGON_MODE,polygon);gl->glGetIntegerv(GL_CURRENT_PROGRAM,&program);
    gl->glGetIntegerv(GL_TEXTURE_BINDING_2D,&texture);gl->glGetIntegerv(GL_DEPTH_FUNC,&depth);
    gl->glGetFloatv(GL_POLYGON_OFFSET_FACTOR,&factor);gl->glGetFloatv(GL_POLYGON_OFFSET_UNITS,&units);
    stream<<polygon[0]<<' '<<program<<' '<<texture<<' '<<depth<<' '
      <<int(gl->glIsEnabled(GL_POLYGON_OFFSET_FILL))<<' '<<int(gl->glIsEnabled(GL_POLYGON_OFFSET_LINE))<<' '
      <<factor<<' '<<units<<' '<<int(gl->glIsEnabled(GL_TEXTURE_2D))<<' '
      <<int(gl->glIsEnabled(GL_TEXTURE_GEN_S))<<' '<<int(gl->glIsEnabled(GL_TEXTURE_GEN_T));
    emit_fixed_texture_state();emit_native_texture_storage(texture);stream<<'\n';
  }
  void emit_fixed_texture_state() {
    if(!context->core11ffp)return;
    GLint environment=0;GLfloat color[4]={},coordinate[4]={};
    context->core11ffp->glGetTexEnviv(GL_TEXTURE_ENV,GL_TEXTURE_ENV_MODE,&environment);
    context->core11fwd->glGetFloatv(GL_CURRENT_COLOR,color);
    context->core11fwd->glGetFloatv(GL_CURRENT_TEXTURE_COORDS,coordinate);
    stream<<' '<<environment<<' '<<int(context->core11fwd->glIsEnabled(GL_TEXTURE_COORD_ARRAY));
    for(float value:color)stream<<' '<<value;
    for(float value:coordinate)stream<<' '<<value;
    emit_native_model_matrix();
  }
  void emit_native_model_matrix() {
    GLfloat matrix[16]={};context->core11fwd->glGetFloatv(GL_MODELVIEW_MATRIX,matrix);
    for(float value:matrix)stream<<' '<<value;
  }
  void emit_native_texture_storage(GLint texture) {
    GLint format=0,mipWidth=0,minFilter=0;
    if(texture!=0) {
      context->core11fwd->glGetTexLevelParameteriv(GL_TEXTURE_2D,0,GL_TEXTURE_INTERNAL_FORMAT,&format);
      context->core11fwd->glGetTexLevelParameteriv(GL_TEXTURE_2D,1,GL_TEXTURE_WIDTH,&mipWidth);
      context->core11fwd->glGetTexParameteriv(GL_TEXTURE_2D,GL_TEXTURE_MIN_FILTER,&minFilter);
    }
    stream<<' '<<format<<' '<<mipWidth<<' '<<minFilter;
  }
};
thread_local NativeCapMeshDrawTrace* NativeCapMeshDrawTrace::active=nullptr;
