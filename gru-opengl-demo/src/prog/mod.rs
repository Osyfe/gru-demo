use gru_opengl::*;
use gru_math::*;
use gru_text::*;

const FRAMEBUFFER_SIZE: u32 = 1024;
const ATLAS_SIZE: u32 = 1024;
const MAX_CHARS: u32 = 60;

const fn cube() -> ([u16; 36], [CubeVertex; 24])
{
    let indices =
    [
        /*
        2, 6, 4, 4, 0, 2, //f
        7, 3, 1, 1, 5, 7, //b
        3, 2, 0, 0, 1, 3, //l
        6, 7, 5, 5, 4, 6, //r
        0, 4, 5, 5, 1, 0, //u
        3, 7, 6, 6, 2, 3  //d
        */
        0, 1, 3, 3, 2, 0, //l
        4, 6, 7, 7, 5, 4, //r
        8, 9, 11, 11, 10, 8, //f
        12, 14, 15, 15, 13, 12, //b
        16, 18, 19, 19, 17, 16, //u
        20, 21, 23, 23, 22, 20 //d
    ];
    let vertices =
    [
        /*
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0] }, // luf 0
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0] }, // lub 1
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0] }, // ldf 2
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0] }, // ldb 3
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0] }, // ruf 4
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0] }, // rub 5
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0] }, // rdf 6
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0] }  // rdb 7
        */ 
        //Left
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //0 luf 0  
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0] }, //1 lub 1
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0] }, //2 ldf 2
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [1.0, 1.0] }, //3 ldb 3
        //Right
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //4 ruf 4
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [0.0, 1.0] }, //5 rub 5
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] }, //6 rdf 6
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },  //7 rdb 7
        //Front
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //8 luf 0
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0] }, //9 ldf 2
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0] }, //10 ruf 4
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [1.0, 1.0] }, //11 rdf 6
        //Back
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0] }, //12 lub 1
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [0.0, 1.0] }, //13 ldb 3
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [1.0, 0.0] }, //14 rub 5
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] } , //15 rdb 7
        //Up
        CubeVertex { position: [-1.0, -1.0, -1.0], color: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] }, //16 luf 0
        CubeVertex { position: [-1.0, -1.0, 1.0], color: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] }, //17 lub 1
        CubeVertex { position: [1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] }, //18 ruf 4
        CubeVertex { position: [1.0, -1.0, 1.0], color: [1.0, 0.0, 1.0], tex_coords: [1.0, 1.0] }, //19 rub 5
        //Down
        CubeVertex { position: [-1.0, 1.0, -1.0], color: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] }, //20 ldf 2
        CubeVertex { position: [-1.0, 1.0, 1.0], color: [0.0, 1.0, 1.0], tex_coords: [1.0, 0.0] }, //21 ldb 3
        CubeVertex { position: [1.0, 1.0, -1.0], color: [1.0, 1.0, 0.0], tex_coords: [0.0, 1.0] }, //22 rdf 6
        CubeVertex { position: [1.0, 1.0, 1.0], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] }  //23 rdb 7
    ];
    (indices, vertices)
}

#[repr(C, packed)]
pub struct GlyphVertex
{
    position: [f32; 2],
    tex_coords: [f32; 2]
}

impl AttributesReprCpacked for GlyphVertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 2 }, "position"),
        (BufferType::Float { size: 2 }, "tex_coords")
    ];
}

#[repr(C, packed)]
pub struct CubeVertex
{
    position: [f32; 3],
    color: [f32; 3],
    tex_coords: [f32; 2],
}

impl AttributesReprCpacked for CubeVertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 3 }, "position"),
        (BufferType::Float { size: 3 }, "color"),
        (BufferType::Float { size: 2 }, "tex_coords")
    ];
}

pub struct Demo
{
    t: f32,
    text_shader: Shader,
    text_vertices: VertexBuffer<GlyphVertex>,
    text_indices: IndexBuffer,
    cube_shader: Shader,
    cube_vertices: VertexBuffer<CubeVertex>,
    cube_indices: IndexBuffer,
    atlas: Atlas,
    atlas_texture: Texture,
    framebuffer: Framebuffer
}

impl App for Demo
{
	fn init(gl: &mut Gl) -> Self
	{
        let text_shader = gl.new_shader(include_str!("../glsl/text.vert"), include_str!("../glsl/text.frag"));
        let text_vertices = gl.new_vertex_buffer(MAX_CHARS * 4, BufferAccess::DYNAMIC);
        let text_indices = gl.new_index_buffer(MAX_CHARS * 6, BufferAccess::DYNAMIC);

        let cube_shader = gl.new_shader(include_str!("../glsl/cube.vert"), include_str!("../glsl/cube.frag"));
        let (indices, vertices) = cube();
        let mut cube_vertices = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::STATIC);
        cube_vertices.data(0, &vertices);
        let mut cube_indices = gl.new_index_buffer(indices.len() as u32, BufferAccess::STATIC);
        cube_indices.data(0, &indices);   

        let font = Font::new(include_bytes!("../res/futuram.ttf"));
        let (font_texture, atlas) = Atlas::new(&font, &Font::all_letters(), 100.0, ATLAS_SIZE, 5);
        if font_texture.len() > 1 { panic!("Atlas more than one page"); }
        let atlas_texture = gl.new_texture(&TextureConfig { size: ATLAS_SIZE, channel: TextureChannel::A, mipmap: false, wrap: TextureWrap::Repeat}, &font_texture[0]);

        let framebuffer = gl.new_framebuffer(&FramebufferConfig { depth: false, size: FRAMEBUFFER_SIZE, wrap: TextureWrap::Clamp });

		Self { t: 0.0, text_shader, text_vertices, text_indices, cube_shader, cube_vertices, cube_indices, atlas, atlas_texture, framebuffer }
	}

    fn input(&mut self, event: event::Event)
    {
    }

    fn frame(&mut self, dt: f32, gl: &mut Gl, window_dims: (u32, u32)) -> bool
    {
        self.t += dt;

        println!();
        let text = "Hello Cube";
        let mut vertices = Vec::with_capacity(text.len() * 4);
        let mut indices = Vec::with_capacity(text.len() * 6);
        let num_lines = 2.0;
        let text_data = self.atlas.text
        (
            text,
            num_lines,
            Align::Center,
            &mut |index| indices.push(index as u16),
            &mut |(s, t, _), (x, y)| vertices.push(GlyphVertex { position: [x, -y], tex_coords: [s, t] })
        );
        self.text_vertices.data(0, &vertices);
        self.text_indices.data(0, &indices);

        gl
            .render_pass(Some(&mut self.framebuffer), RenderPassInfo { clear_color: Some((1.0, 1.0, 1.0)), clear_depth: false })
            .pipeline(&self.text_shader, PipelineInfo { depth_test: false, alpha_blend: true, face_cull: true })
            .uniform_f1("height", 2.0 / num_lines)
            .uniform_texture("atlas", &self.atlas_texture, false)
            .draw(&self.text_vertices, Some(&self.text_indices), 0, text_data.index_count);

        let z = -(self.t.sin() / 2.0 * 9.0 * 1.0 + 5.5);
        let mat = 
            Mat4::perspective_opengl(window_dims.0 as f32 / window_dims.1 as f32, std::f32::consts::FRAC_PI_8, 1.0, 10.0)
          * Mat4::translation_z(-8.0)
          * Mat4::rotation(Vec3(1.0, 1.0, 1.0).unit(), self.t);
        gl
            .render_pass(None, RenderPassInfo { clear_color: Some((0.2, 0.1, 0.8)), clear_depth: true })
            .pipeline(&self.cube_shader, PipelineInfo { depth_test: true, alpha_blend: false, face_cull: true })
            .uniform_mat4("mat", mat)
            .uniform_texture("tex", self.framebuffer.texture(), false)
            .draw(&self.cube_vertices, Some(&self.cube_indices), 0, 36);
        true
    }
}
