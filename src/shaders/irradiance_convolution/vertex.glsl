#version 330 core
in vec3 position;

out vec3 v_position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    v_position = position;
    gl_Position = projection * view * vec4(v_position, 1.0);
}
