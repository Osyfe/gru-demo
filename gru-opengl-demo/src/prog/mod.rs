use gru_opengl::*;
use gru_math::*;

#[repr(C, packed)]
pub struct Vertex
{
    pos: [f32; 2]
}

impl AttributesReprCpacked for Vertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] = &[(BufferType::Float { size: 2 }, "position")];
}

pub struct Demo
{
    t: f32,
    shader: Shader,
    vertices: VertexBuffer<Vertex>
}

impl App for Demo
{
	fn init(gl: &mut Gl) -> Self
	{
        let vertex_shader = r#"
            attribute vec2 position;

            uniform mat2 rot;
            
            void main()
            {
                gl_Position = vec4(rot * position - 0.5, 0.0, 1.0);
            }
        "#;
        let fragment_shader = r#"
            uniform float col;

            void main()
            {
                gl_FragColor = vec4(vec3(col), 1.0);
            }
        "#;
        let shader = gl.new_shader(vertex_shader, fragment_shader);
        let mut vertices = gl.new_vertex_buffer(3, BufferAccess::STATIC);
        vertices.data(0, &[Vertex { pos: [0.5, 1.0] }, Vertex { pos: [0.0, 0.0] }, Vertex { pos: [1.0, 0.0] }]);
		Self { t: 0.0, shader, vertices }
	}

    fn input(&mut self, event: Event)
    {

    }

    fn frame(&mut self, dt: f32, gl: &mut Gl, aspect: f32)
    {
        self.t += dt;
        let mut render_pass = gl.render_pass(RenderPassInfo { clear_color: Some((0.0, 0.0, 0.0)), clear_depth: false });
        let mut pipeline = render_pass.pipeline(&self.shader, PipelineInfo { depth_test: false, alpha_blend: false, face_cull: true });
        pipeline.uniform_f1("col", self.t.sin() / 2.0 + 0.5);
        pipeline.uniform_mat2("rot", Mat2::rotation(self.t / 2.0));
        pipeline.draw(&self.vertices, None, 0, 3);
    }
}
