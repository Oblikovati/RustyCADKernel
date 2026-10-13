#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
layout(location=0) in vec4 vertexColor;
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
#include "oit.glsl"
#else
#include "native_fragment_alpha.glsl"
layout(location=0) out vec4 outColor;
#endif
void main() {
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
    if(oitEarlyReturn())return;
#endif
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif
#if defined(MARTENSITE_OIT_WEIGHTED) || defined(MARTENSITE_OIT_PEELING)
    oitWriteColor(vertexColor);
#else
 outColor = vertexColor;
    nativePostAlpha(outColor.a);
#endif
}
