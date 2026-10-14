#pragma once
#include <OpenGl_GlCore45.hxx>
// Query actual attachment storage without changing texture/renderbuffer bindings or native state.
struct NativeStencilAttachment {
  const OpenGl_Context* context;
  explicit NativeStencilAttachment(const occ::handle<OpenGl_Context>& ctx):context(ctx.get()) {}
  int query(GLenum attachment,GLenum parameter) const {
    GLint value=0;
    context->core30->glGetFramebufferAttachmentParameteriv(GL_DRAW_FRAMEBUFFER,attachment,parameter,&value);
    return value;
  }
  int internal_format(int kind,int name,int level) const {
    if(context->core45==nullptr)return -1;
    GLint value=-1;
    if(kind==GL_TEXTURE)context->core45->glGetTextureLevelParameteriv(name,level,GL_TEXTURE_INTERNAL_FORMAT,&value);
    if(kind==GL_RENDERBUFFER)context->core45->glGetNamedRenderbufferParameteriv(name,GL_RENDERBUFFER_INTERNAL_FORMAT,&value);
    return value;
  }
  int storage_property(int kind,GLenum attachment,GLenum parameter) const {
    // An absent attachment has no storage properties; querying them would inject GL errors.
    if(kind==GL_NONE)return -1;
    return query(attachment,parameter);
  }
  void emit(std::ostream& output,int framebuffer) const {
    const GLenum attachment=framebuffer==0?GL_STENCIL:GL_STENCIL_ATTACHMENT;
    const int kind=query(attachment,GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
    const int name=kind==GL_TEXTURE||kind==GL_RENDERBUFFER?query(attachment,GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME):0;
    const int level=kind==GL_TEXTURE?query(attachment,GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL):0;
    output<<",\"attachment_kind\":"<<kind<<",\"attachment_name\":"<<name
      <<",\"attachment_level\":"<<level<<",\"attachment_internal_format\":"<<internal_format(kind,name,level)
      <<",\"attachment_stencil_size\":"<<storage_property(kind,attachment,GL_FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE)
      <<",\"attachment_component_type\":"<<storage_property(kind,attachment,GL_FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE);
  }
};
