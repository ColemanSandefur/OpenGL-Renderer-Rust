#version 150

in vec3 position;
in mat4 model;
in vec3 normal;

out vec3 v_normal;
out vec3 v_frag_pos;

uniform mat4 view;
uniform mat4 projection;

void main() {
    vec4 pos = (view * model * vec4(position, 1.0));
    v_normal = mat3(transpose(inverse(view * model))) * normal;
    v_frag_pos = pos.xyz;
    
    gl_Position = projection * pos;
}
