#version 330 core
in vec3 position;
in vec2 tex_coords;

out vec2 TexCoords;

void main()
{
    TexCoords = tex_coords;
	gl_Position = vec4(position, 1.0);
}