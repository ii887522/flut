#version 460 core

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 color;

layout(binding = 0) uniform sampler2D atlasSampler;

void main() {
  color = vec4(fragColor.rgb, fragColor.a * texture(atlasSampler, fragTexCoord).r);
}
