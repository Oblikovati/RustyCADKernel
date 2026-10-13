#version 450
#extension GL_GOOGLE_include_directive : require
#define MARTENSITE_FRAGMENT
#include "clipping.glsl"
#include "native_fragment_alpha.glsl"
layout(location=0) in vec4 vertexColor;
layout(location=1) in vec2 textureUV;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0,std430) readonly buffer Sprite { vec4 pixels[]; } sprite;
layout(push_constant) uniform Transform { mat4 clip; uint width; uint height; float cutoff; } transform;
vec4 texel(ivec2 index) {
    ivec2 clipped = clamp(index, ivec2(0), ivec2(transform.width, transform.height) - 1);
    return sprite.pixels[clipped.y * transform.width + clipped.x];
}
void main() {
#ifdef MARTENSITE_CLIPPING
    discardClippedFragment();
#endif

    // FreeType atlases follow native linear sampling; texel centers are half-integers.
    vec2 coordinate = textureUV * vec2(transform.width, transform.height) - vec2(0.5);
    ivec2 origin = ivec2(floor(coordinate));
    vec2 fraction = fract(coordinate);
    vec4 top = mix(texel(origin), texel(origin + ivec2(1,0)), fraction.x);
    vec4 bottom = mix(texel(origin + ivec2(0,1)), texel(origin + ivec2(1,1)), fraction.x);
    outColor = vertexColor * mix(top, bottom, fraction.y);
    if (outColor.a <= transform.cutoff) discard;
    nativePostAlpha(outColor.a);
}
