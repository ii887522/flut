#version 460 core

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragTexCoord;

layout(location = 0) out vec4 color;
layout(depth_greater) out float gl_FragDepth;

layout(set = 0, binding = 0) uniform sampler texSampler;
layout(set = 0, binding = 1) uniform texture2D textures[2];

void main() {
  const float alpha = fragColor.a * textureLod(sampler2D(textures[int(fragTexCoord.z)], texSampler), fragTexCoord.xy, 0).r;
  color = vec4(fragColor.rgb, alpha);

  if (alpha >= 0.5) {
    gl_FragDepth = gl_FragCoord.z;
  } else {
    gl_FragDepth = 1.0;
  }
}
