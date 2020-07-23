#version 150 core

in vec2 v_TexCoord;
in vec4 v_Color;

out vec4 finalColor;

void main() {
    finalColor = v_Color;
}
