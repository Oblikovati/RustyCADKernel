// Shared nearest-sampled sprite coverage preserves top-down native marker rows.
#ifndef MARTENSITE_SPRITE_SET
#define MARTENSITE_SPRITE_SET 0
#endif
layout(set=MARTENSITE_SPRITE_SET,binding=0,std430) readonly buffer Sprite { vec4 pixels[]; } sprite;
vec4 markerTexel(uvec2 extent) {
    uvec2 pixel = min(uvec2(gl_PointCoord * vec2(extent)), extent - 1u);
    return sprite.pixels[pixel.y * extent.x + pixel.x];
}

// OCCT 3d097a0 sprite coverage replaces alpha; plain POINT retains vertex alpha.
vec4 unlitMarkerColor(vec4 texel, vec4 vertex, uint policy) {
    if (policy == 4u) return vertex;
#ifdef MARTENSITE_FIXED_LIGHTING
    if (policy == 2u || policy == 3u) return texel;
#else
    if (policy == 2u) return texel;
#endif
    if (policy == 1u || policy == 3u) return vec4(vertex.rgb, texel.a);
    return texel * vertex;
}
vec4 litMarkerColor(vec4 texel, vec4 lit, uint policy) {
    if (policy == 4u) return lit;
    if (policy == 1u) return vec4(lit.rgb, texel.a);
#ifdef MARTENSITE_FIXED_LIGHTING
    // applyGlobalTextureParams overrides DECAL with REPLACE for GenMode SPRITE.
    return texel;
#else
    return texel * lit;
#endif
}
