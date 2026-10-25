#version 450

layout(push_constant) uniform PushBlock {
    mat4 model;
} push;

layout(location = 0) in vec3 inPos;    // 位置，对应 Vertex.pos
layout(location = 1) in vec3 inColor;  // 颜色，对应 Vertex.color


layout(location = 0) out vec3 fragColor;

void main() {
    
    gl_Position = push.model * vec4(inPos, 1.0);
    fragColor = inColor;
}