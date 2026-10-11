#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=0) out vec4 outColor;
#include "marker.glsl"
layout(push_constant) uniform Transform { mat4 clip; float size; uint width; uint height; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif

    outColor = markerTexel(uvec2(transform.width, transform.height)) * vertexColor;
    if (outColor.a <= 0.1) discard;
}
