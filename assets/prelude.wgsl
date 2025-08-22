#define_import_path smud::prelude

const PI: f32 = 3.141592653589793;

// Input struct for SDF functions containing all available data
struct SdfInput {
    pos: vec2<f32>,      // Position in shape space
    params: vec4<f32>,   // User-defined parameters
}