#version 450
#extension GL_GOOGLE_include_directive : require
#include "clipping.glsl"
layout(location=0) in vec4 position;
layout(location=1) in vec4 color;
layout(location=0) out vec4 vertexColor;
layout(push_constant) uniform Transform { mat4 clip; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    clippingWorldPosition = clipping.worldFromObject * position;
#endif

    gl_Position = transform.clip * position;
    // Graphic3d cameras use OpenGL clip depth; Vulkan uses zero-to-one depth.
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
    vertexColor = color;
}
