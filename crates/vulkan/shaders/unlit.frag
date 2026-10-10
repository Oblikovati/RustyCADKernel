#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=0) out vec4 outColor;
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif
 outColor = vertexColor; }
