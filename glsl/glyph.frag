#version 460

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 color;

layout(binding = 0) uniform sampler2D texSampler;

void main() {
  color = vec4(fragColor.rgb, fragColor.a * textureLod(texSampler, fragTexCoord, 0).r);
}
