#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0,std430) readonly buffer Sprite { vec4 pixels[]; } sprite;
layout(push_constant) uniform Transform { mat4 clip; float size; uint width; uint height; } transform;
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif

    // Logical pixmap rows and Vulkan point coordinates both run from top to bottom.
    uvec2 extent = uvec2(transform.width, transform.height);
    uvec2 pixel = min(uvec2(gl_PointCoord * vec2(extent)), extent - 1u);
    vec4 texel = sprite.pixels[pixel.y * extent.x + pixel.x];
    outColor = texel * vertexColor;
    if (outColor.a <= 0.1) discard;
}
