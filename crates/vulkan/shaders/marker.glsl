// Shared nearest-sampled sprite coverage preserves top-down native marker rows.
#ifndef MARTENSITE_SPRITE_SET
#define MARTENSITE_SPRITE_SET 0
#endif
layout(set=MARTENSITE_SPRITE_SET,binding=0,std430) readonly buffer Sprite { vec4 pixels[]; } sprite;
vec4 markerTexel(uvec2 extent) {
    uvec2 pixel = min(uvec2(gl_PointCoord * vec2(extent)), extent - 1u);
    return sprite.pixels[pixel.y * extent.x + pixel.x];
}
