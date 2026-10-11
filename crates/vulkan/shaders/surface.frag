#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
layout(location=0) in vec4 vertexColor;
layout(location=7) in vec4 surfaceUV;
#ifdef MARTENSITE_SURFACE_TEXTURE
#ifdef MARTENSITE_LIGHTING
#define MARTENSITE_SURFACE_SET 2
#else
#define MARTENSITE_SURFACE_SET 0
#endif
layout(set=MARTENSITE_SURFACE_SET,binding=0) uniform sampler2D surfaceImage;
vec4 surfaceTexel() {return texture(surfaceImage,surfaceUV.st/surfaceUV.w);}
#endif
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
layout(push_constant) uniform Transform {mat4 clip;float cutoff;uint flags;} transform;
vec4 sampledSurfaceColor() {
#ifdef MARTENSITE_LIGHTING
    vec4 result=litFragmentColor();
#ifdef MARTENSITE_SURFACE_TEXTURE
    if (lighting.settings.x==1u) result*=surfaceTexel();
#endif
    return result;
#elif defined(MARTENSITE_SURFACE_TEXTURE)
    bool useVertex=(transform.flags&2u)!=0u&&(gl_FrontFacing||(transform.flags&4u)!=0u);
    return useVertex?vertexColor:surfaceTexel();
#else
    return vertexColor;
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
