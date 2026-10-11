"""Point cloud geometry, whole-object and subset-owner capture controls."""
POINT_CLOUD_MODES = (
    "cloud-points", "cloud-colors", "cloud-normals-unlit", "cloud-box",
    "cloud-lit-phong", "cloud-lit-gouraud", "cloud-lit-colors", "cloud-lit-clipped", "cloud-lit-normal-updated",
    "cloud-selected", "cloud-subset-selected", "cloud-subset-detected",
    "cloud-moved", "cloud-erased", "cloud-redisplayed", "cloud-unset-color",
)


def point_cloud_pixels(width: int, height: int, rgba: bytes, mode: str) -> dict[str, object]:
    """Reject blank geometry or missing subset overlays; point_cloud_pixels(1,1,bytes([0,255,255,255]),'cloud-subset-detected')."""
    if width <= 0 or height <= 0 or len(rgba) != width * height * 4:
        raise ValueError(f"received {width=} {height=} RGBA length {len(rgba)}; expected positive dimensions and four bytes per pixel")
    pixels = [tuple(rgba[i:i+4]) for i in range(0, len(rgba), 4)]
    visible = sum(pixel != (0, 0, 255, 255) for pixel in pixels)
    expected = (0, 255, 255, 255) if mode == "cloud-subset-detected" else (255, 255, 0, 255)
    subset = sum(pixel == expected for pixel in pixels) if "subset" in mode else None
    passed = visible == 0 if mode == "cloud-erased" else visible > 0 and (subset is None or subset > 0)
    return {"status": "passed" if passed else "failed", "visible_pixels": visible, "subset_pixels": subset}
