#version 460 core

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 uv;

layout(binding = 0) uniform sampler2D atlas_sampler;

layout(location = 0) out vec4 out_color;

void main() {
  out_color = vec4(color.rgb, color.a * texture(atlas_sampler, uv).r);
}
