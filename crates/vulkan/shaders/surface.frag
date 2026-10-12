#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#define MARTENSITE_SURFACE_FAMILY
layout(push_constant) uniform Transform {mat4 clip;float cutoff;uint flags;} transform;
layout(location=0) in vec4 vertexColor;
layout(location=7) in vec4 surfaceUV;
#ifdef MARTENSITE_SURFACE_TEXTURE
#ifdef MARTENSITE_LIGHTING
#define MARTENSITE_SURFACE_SET 2
#else
#define MARTENSITE_SURFACE_SET 0
#endif
layout(set=MARTENSITE_SURFACE_SET,binding=0) uniform sampler2D surfaceImage;
#endif
vec4 surfaceTexel() {
    // OpenGL incomplete texture sampling is black with alpha one; loading still failed.
    if ((transform.flags&16u)!=0u) return vec4(0.0,0.0,0.0,1.0);
#ifdef MARTENSITE_SURFACE_TEXTURE
    return texture(surfaceImage,surfaceUV.st/surfaceUV.w);
#else
    return vec4(1.0);
#endif
}
#ifdef MARTENSITE_LIGHTING
#include "lighting.glsl"
layout(location=1) in vec3 worldNormal;
layout(location=2) in vec3 viewDirection;
layout(location=4) in vec4 worldPosition;
layout(location=5) in vec4 frontColor;
layout(location=6) in vec4 backColor;
#include "lit_fragment.glsl"
#endif
#include "clipping.glsl"
layout(location=0) out vec4 outColor;
vec4 sampledSurfaceColor() {
#ifdef MARTENSITE_LIGHTING
    vec4 result=litFragmentColor();
    if (lighting.settings.x==1u) result*=surfaceTexel();
    return result;
#else
    bool useVertex=(transform.flags&2u)!=0u&&(gl_FrontFacing||(transform.flags&4u)!=0u);
    return useVertex?vertexColor:((transform.flags&17u)!=0u?surfaceTexel():vertexColor);
#endif
}
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif
    outColor=sampledSurfaceColor();
    // Surface masking uses native strict comparison; glyph cutoff remains <= in its independent shader.
    if ((transform.flags&8u)!=0u&&outColor.a<transform.cutoff) discard;
}
