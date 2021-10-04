#version 450

/* ---- INPUT ---- */
// Vertex Data
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;

// Instance Data
layout(location = 3) in mat4 world;

// Uniform Data
layout(set = 0, binding = 0) uniform Data {
  mat4 projection;
  mat4 view;
}
uniforms;

/* ---- OUTPUT ---- */
layout(location = 0) out vec3 view_position;
layout(location = 1) out vec3 view_normal;
layout(location = 2) out vec4 vert_color;

/* ---- MAIN ---- */
void main() {
  mat4 world_view = uniforms.view * world;
  mat4 normal_view = transpose(inverse(world_view));

  view_position = (world_view * vec4(position, 1.0)).xyz;
  view_normal = (normal_view * vec4(normal, 0.0)).xyz;
  vert_color = color;

  gl_Position = uniforms.projection * vec4(view_position, 1.0);
}