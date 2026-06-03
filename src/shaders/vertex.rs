pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 0) out vec3 v_color;
            layout(set = 0, binding = 0) uniform Camera {
                mat4 world_to_clip;
            } camera;

            void main() {
                gl_Position = camera.world_to_clip * vec4(position, 1.0);
                v_color = color;
            }
        ",
    }
}
