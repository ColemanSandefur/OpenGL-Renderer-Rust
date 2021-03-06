#version 330 core
in vec3 position;
in vec3 normal;
in vec2 tex_coords;

out vec2 v_tex_coords;
out vec3 v_world_pos;
out vec3 v_normal;

uniform mat4 model;
uniform mat4 projection;
uniform mat4 view;

void main() {
    v_tex_coords = tex_coords;
    v_world_pos = vec3(model * vec4(position, 1.0));
    v_normal = mat3(model) * normal;

    gl_Position = projection * view * vec4(v_world_pos, 1.0);
}
