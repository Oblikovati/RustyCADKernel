# Use acadrust for Martensite DWG support

On 2026-10-12, the user selected [acadrust](https://github.com/hakanaktt/acadrust)
as the backend for Martensite's DWG format support. Integrate its DWG reader and
writer behind a thin Martensite-owned interface, with drawing conversion and
behavior covered by Rust tests. Schedule this integration with exchange-format
work, after the current visualization and Vulkan renderer priorities.
