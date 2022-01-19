#version 150
in vec3 position;
in vec3 normal;
in mat4 model;

out vec3 v_normal;
out vec3 v_position;

uniform mat4 projection;
uniform mat4 view;
void main() {
    mat4 modelview = view * model;
    v_normal = transpose(inverse(mat3(modelview))) * normal;
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_position = gl_Position.xyz / gl_Position.w;
}
