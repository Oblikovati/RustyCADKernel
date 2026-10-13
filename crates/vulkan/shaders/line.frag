#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
#include "native_fragment_alpha.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=1) in vec2 screenSpaceCoord;
layout(location=0) out vec4 fragmentColor;
layout(push_constant) uniform LineTransform { mat4 clip; vec2 viewport; uint pattern; uint factor; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif

#ifndef MARTENSITE_FIXED_LINE_STIPPLE
    // OCCT 3d097a0 Graphic3d_ShaderManager uses the dominant screen derivative,
    // not a stipple counter restarted at each segment. Convert Vulkan's Y origin.
    float coordinate = transform.viewport.y - gl_FragCoord.y;
    if (abs(dFdx(screenSpaceCoord.x)) - abs(dFdy(screenSpaceCoord.y)) > 0.001)
        coordinate = gl_FragCoord.x;
    uint bit = uint(floor(coordinate / float(transform.factor) + 0.5)) & 15u;
    if ((transform.pattern & (1u << bit)) == 0u) discard;
#endif
    fragmentColor = vertexColor;
    nativePostAlpha(fragmentColor.a);
}
