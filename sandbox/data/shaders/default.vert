#version 450
#extension GL_ARB_separate_shader_objects : enable

/* layout(binding = 0) uniform UniformBufferObject { */
/*     mat4 model; */
/*     mat4 view; */
/*     mat4 proj; */
/* } ubo; */

layout(push_constant) uniform Transform {
        mat4 mvp;
} transform;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;

layout(location = 0) out vec2 fragTexCoord;

void main() {
    gl_Position = transform.mvp * vec4(inPosition, 1.0);
    fragTexCoord = inTexCoord;
}
