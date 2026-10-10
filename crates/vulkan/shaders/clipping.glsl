// OCCT 3d097a0 Graphic3d_ShaderManager: union within each chain, intersection between chains.
#ifdef MARTENSITE_CLIPPING
struct ClipHalfspace { vec4 equation; uvec4 chain; };
layout(set=1,binding=0,std430) readonly buffer ClipVolume {
    mat4 worldFromObject;
    uvec4 count;
    ClipHalfspace planes[];
} clipping;
#ifdef MARTENSITE_FRAGMENT
layout(location=3) in vec4 clippingWorldPosition;
void discardClippedFragment() {
    vec3 point = clippingWorldPosition.xyz / clippingWorldPosition.w;
    for (uint index = 0u; index < clipping.count.x;) {
        vec4 equation = clipping.planes[index].equation;
        uint remaining = clipping.planes[index].chain.x;
        if (dot(equation.xyz, point) + equation.w < 0.0) {
            if (remaining == 1u) discard;
            index += 1u;
        } else {
            index += remaining;
        }
    }
}
#else
layout(location=3) out vec4 clippingWorldPosition;
#endif
#endif
