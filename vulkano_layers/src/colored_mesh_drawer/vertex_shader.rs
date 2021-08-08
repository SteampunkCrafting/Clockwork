vulkano_shaders::shader! {
    ty: "vertex",
    src: "
#version 450

/* ---- INPUT ---- */
// Vertex Data
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;

// Instance Data
layout (location = 3) in mat4 world;

// Uniform Data
layout(set = 0, binding = 0) uniform Data {
    mat4 projection;
    mat4 view;
} uniforms;

/* ---- OUTPUT ---- */
// Color
layout (location = 0) out vec4 vert_color;

/* ---- MAIN ---- */
void main() {
    mat4 world_view = uniforms.view * world;
    gl_Position = uniforms.projection * world_view * vec4(position, 1.0);
    vert_color = color;
}
        "
}
