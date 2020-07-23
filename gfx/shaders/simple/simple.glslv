#version 150 core

in vec2 a_Pos;
in vec2 a_TexCoord;
in vec4 a_Color;

out vec4 v_Color;
out vec2 v_TexCoord;

uniform Locals {
    mat4 u_Transform;
};

void main() {
    v_Color     = vec4(a_Color[0]/255.0, a_Color[1]/255.0, a_Color[2]/255.0, a_Color[3]/255.0);
    v_TexCoord  = a_TexCoord;
    gl_Position = vec4(a_Pos, 0.0, 1.0) * u_Transform;
}
