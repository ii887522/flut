#version 460 core

layout(location = 0) in vec4 color;
layout(location = 1) in vec2 local_pos;
layout(location = 2) in vec2 half_size;
layout(location = 3) in float radius;

layout(location = 0) out vec4 out_color;

void main() {
  // Signed distance for a rounded box 2D.
  // Reference: https://iquilezles.org/articles/distfunctions2d/
  const float d = length(max(abs(local_pos) - half_size + vec2(radius), vec2(0.0))) - radius;

  // Anti-aliasing for smoother edges
  const float alpha = 1.0 - smoothstep(0.0, 1.5, d / fwidth(d));
  out_color = vec4(color.rgb, color.a * alpha);
}
