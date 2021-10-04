#version 450

/* ---- INPUT ---- */
// Vertex Data
layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texture_coord;

// Instance Data
layout(location = 2) in mat4 world;

// Uniform Data
layout(set = 0, binding = 0) uniform Data {
  mat4 projection;
  mat4 view;
}
uniforms;

/* ---- OUTPUT ---- */
layout(location = 0) out vec2 vert_texture;

/* ---- MAIN ---- */
void main() {
  vert_texture = texture_coord;
  gl_Position = uniforms.projection *
                vec4((uniforms.view * world * vec4(position, 0.0)).xyz, 1);
}