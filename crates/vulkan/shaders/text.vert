#version 450
#extension GL_GOOGLE_include_directive : require
#include "clipping.glsl"
layout(location=0) in vec4 position;
layout(location=1) in vec4 color;
layout(location=2) in vec2 uv;
layout(location=0) out vec4 vertexColor;
layout(location=1) out vec2 textureUV;
layout(push_constant) uniform Transform { mat4 clip; uint width; uint height; float cutoff; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    clippingWorldPosition = clipping.worldFromObject * position;
#endif

    gl_Position = transform.clip * position;
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
    vertexColor = color;
    textureUV = uv;
}
