// OCCT 3d097a0: world-space Phong family, including legacy intensity and cone quirks.
struct RenderLight { uvec4 kind; vec4 color; vec4 position; vec4 direction; vec4 params; };
layout(set=0,binding=0,std430) readonly buffer SurfaceLighting {
    mat4 worldFromObject;
    mat4 normalFromObject;
    vec4 eye;
    vec4 ambient;
    vec4 materials[8];
    uvec4 settings;
    RenderLight sources[];
} lighting;
float pointAttenuation(float distance, RenderLight light) {
    float range = light.direction.w;
    if (range == 0.0) return 1.0 / (light.params.x + light.params.y * distance);
    if (distance > range) return -1.0;
    float ratio = distance / range;
    ratio *= ratio;
    ratio *= ratio;
    return clamp(1.0 - ratio, 0.0, 1.0) / max(0.0001, distance * distance);
}
float spotAttenuation(vec3 direction, RenderLight light) {
    float cosine = dot(light.direction.xyz, -direction);
    // Native exact-axis spot lights are dark; retain this observable behavior.
    if (cosine >= 1.0 || cosine < cos(light.params.z)) return 0.0;
    return light.params.w > 0.0 ? pow(cosine, light.params.w * 128.0) : 1.0;
}
vec4 computeLighting(vec3 normal, vec3 view, vec4 world, bool front, vec4 vertexColor) {
    uint face = front ? 0u : 4u;
    vec3 diffuse = vec3(0.0), specular = vec3(0.0);
    vec3 faceNormal = front ? normal : -normal;
    for (uint i=0u; i<lighting.settings.y; ++i) {
        RenderLight light = lighting.sources[i];
        vec3 direction = light.position.xyz;
        float attenuation = 1.0;
        if (light.kind.x != 1u) {
            direction -= world.xyz / world.w;
            float distance = length(direction);
            attenuation = pointAttenuation(distance, light);
            if (attenuation <= 0.0) continue;
            direction /= distance;
            if (light.kind.x == 3u) attenuation *= spotAttenuation(direction, light);
        }
        float ndotl = max(0.0, dot(faceNormal, direction));
        float ndoth = max(0.0, dot(faceNormal, normalize(direction + view)));
        float shine = ndotl > 0.0 ? pow(ndoth, lighting.materials[face+2u].w) : 0.0;
        diffuse += light.color.rgb * ndotl * attenuation;
        specular += light.color.rgb * shine * attenuation;
    }
    vec4 result = vec4(lighting.ambient.rgb * lighting.materials[face+3u].rgb
        + diffuse * lighting.materials[face].rgb + specular * lighting.materials[face+2u].rgb,
        lighting.materials[face].a);
    if (lighting.settings.z != 0u && (front || lighting.settings.w == 0u)) result *= vertexColor;
#if defined(MARTENSITE_FRAGMENT) && defined(MARTENSITE_SURFACE_TEXTURE)
    // Native Phong modulates the lit terms before adding untextured emission; Gouraud modulates later.
    result *= surfaceTexel();
#endif
    result.rgb += lighting.materials[face+1u].rgb;
    return result;
}
