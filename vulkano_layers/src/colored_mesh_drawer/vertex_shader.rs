vulkano_shaders::shader! {
    ty: "vertex",
    src: "
#version 450

// Input (Vertex Data)
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;

// Input (Instance Data)
layout (location = 3) in mat4 transformation;

// Output (Color)
layout (location = 0) out vec4 vert_color;

void main() {
    gl_Position = transformation * vec4(position, 1.0);
    vert_color = color;
}
        "
}
