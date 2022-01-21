#version 330
in vec3 position;
in vec3 normal;
in mat4 model;

uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

out vec3 v_normal;
out vec3 v_position;
flat out Material v_material;

uniform mat4 projection;
uniform mat4 view;

void main() {
    v_material = Material (
        ambient, diffuse, specular, shininess
    );
    mat4 modelview = view * model;
    v_normal = transpose(inverse(mat3(modelview))) * normal;
    gl_Position = projection * modelview * vec4(position, 1.0);
    v_position = gl_Position.xyz / gl_Position.w;
}
