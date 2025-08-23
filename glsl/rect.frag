#version 460 core
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragTexCoord;

layout(location = 0) out vec4 color;

layout(binding = 0) uniform sampler2D atlasSamplers[];

void main() {
  color = vec4(fragColor.rgb, fragColor.a * texture(nonuniformEXT(atlasSamplers[uint(fragTexCoord.z)]), fragTexCoord.xy).r);
}
