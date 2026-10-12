vec4 litFragmentColor() {
    // OCCT 3d097a0 line/marker shaders always evaluate the front material.
#if defined(MARTENSITE_LINE_LIGHTING) || defined(MARTENSITE_MARKER_LIGHTING)
    bool frontFacing = true;
#else
    bool frontFacing = gl_FrontFacing;
#endif
    if (lighting.settings.x == 1u) {
        return frontFacing ? frontColor : backColor;
    }
    vec3 normal = normalize(worldNormal);
    if (lighting.settings.x == 0u) {
        vec3 point = worldPosition.xyz / worldPosition.w;
        // Negative Vulkan viewport reverses screen Y relative to OpenGL derivatives.
        normal = normalize(-cross(dFdx(point), dFdy(point)));
        normal = gl_FrontFacing ? normal : -normal;
    }
    return computeLighting(normal, normalize(viewDirection), worldPosition, frontFacing, vertexColor);
}

