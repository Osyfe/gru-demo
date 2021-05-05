use std::collections::HashSet;
use gru_opengl::*;
use gru_math::*;
use gru_text::*;

mod text;
mod cube;
use text::*;
use cube::*;

const FRAMEBUFFER_SIZE: u32 = 1024;
const TARGET_ROT: Vec3 = Vec3(0.5, 0.5, 0.5);

struct TextData
{
    shader: Shader,
    vertices: VertexBuffer<GlyphVertex>,
    indices: IndexBuffer,
    text: Text,
    index_count: u32,
    height_key: UniformKey,
    atlas_key: UniformKey
}

struct CubeData
{
    shader: Shader,
    vertices: VertexBuffer<CubeVertex>,
    indices: IndexBuffer,
    mat_key: UniformKey,
    tex_key: UniformKey
}

struct AtlasData
{
    atlas: Atlas,
    texture: Texture
}

struct InputData
{
    last_pos: (f32, f32),
    mouse_down: bool
}

pub struct Demo
{
    text: TextData,
    cube: CubeData,
    atlas: AtlasData,
    framebuffer: Framebuffer,
    rot: Rotor,
    vel: Vec3,
    input: InputData
}

impl App for Demo
{
	fn init(gl: &mut Gl) -> Self
	{
        let max_chars = TEXTS.iter().map(|text| text.chars().count() as u32).max().unwrap();

        let text_shader = gl.new_shader(include_str!("../glsl/text.vert"), include_str!("../glsl/text.frag"));
        let text_vertices = gl.new_vertex_buffer(max_chars * 4, BufferAccess::DYNAMIC);
        let text_indices = gl.new_index_buffer(max_chars * 6, BufferAccess::DYNAMIC);
        let height_key = text_shader.get_key("height").unwrap().clone();
        let atlas_key = text_shader.get_key("atlas").unwrap().clone();

        let cube_shader = gl.new_shader(include_str!("../glsl/cube.vert"), include_str!("../glsl/cube.frag"));
        let (indices, vertices) = cube();
        let mut cube_vertices = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::STATIC);
        cube_vertices.data(0, &vertices);
        let mut cube_indices = gl.new_index_buffer(indices.len() as u32, BufferAccess::STATIC);
        cube_indices.data(0, &indices);   
        let mat_key = cube_shader.get_key("mat").unwrap().clone();
        let tex_key = cube_shader.get_key("tex").unwrap().clone();

        let mut alphabet = HashSet::new();
        for text in &TEXTS { for c in text.chars() { if c != ' ' { alphabet.insert(c); } } }
        let font = Font::new(include_bytes!("../res/futuram.ttf"));
        let (font_texture, atlas) = Atlas::new(&font, &alphabet, 100.0, ATLAS_SIZE, 5);
        if font_texture.len() > 1 { panic!("Atlas more than one page"); }
        let atlas_texture = gl.new_texture(&TextureConfig { size: ATLAS_SIZE, channel: TextureChannel::A, mipmap: false, wrap: TextureWrap::Repeat}, &font_texture[0]);

        let framebuffer = gl.new_framebuffer(&FramebufferConfig { depth: false, size: FRAMEBUFFER_SIZE, wrap: TextureWrap::Clamp });

        let input = InputData
        {
            last_pos: (0.0, 0.0),
            mouse_down: false
        };

		Self
        {
            text: TextData
            {
                shader: text_shader,
                vertices: text_vertices, indices: text_indices,
                text: Text::None, index_count: 0,
                height_key, atlas_key
            },
            cube: CubeData
            {
                shader: cube_shader,
                vertices: cube_vertices, indices: cube_indices,
                mat_key, tex_key
            },
            atlas: AtlasData
            {
                atlas,
                texture: atlas_texture
            },
            framebuffer,
            rot: Rotor::identity(),
            vel: TARGET_ROT,
            input
        }
	}

    fn input(&mut self, event: event::Event)
    {
        use event::*;
        match event
        {
            Event::Click { button: MouseButton::Left, state } =>
            {
                self.input.mouse_down = state == ElementState::Pressed
            },
            Event::Cursor { position } =>
            {
                let (x, y) = position;
                let (x2, y2) = self.input.last_pos;
                if self.input.mouse_down
                {
                    let diff = Vec3(y2 - y, x - x2, 0.0);
                    let vel = 0.005 * diff.norm().sqrt() + 0.005;
                    self.vel += diff * vel;
                }
                self.input.last_pos = position;
            },
            Event::Touch { position, phase, .. } =>
            {
                let (x, y) = position;
                if let TouchPhase::Moved = phase
                {
                    let (x2, y2) = self.input.last_pos;
                    let diff = Vec3(y2 - y, x - x2, 0.0);
                    let vel = 0.005 * diff.norm().sqrt() + 0.005;
                    self.vel += diff * vel;
                }
                self.input.last_pos = position;
            }
            _ => {}
        }
    }

    fn frame(&mut self, dt: f32, gl: &mut Gl, window_dims: (u32, u32)) -> bool
    {
        //physic
        self.vel += (TARGET_ROT - self.vel) * dt;
        self.rot = Rotor::from_axis(self.vel * dt) * self.rot;
        //text
        let current_text = Text::Hello;
        let (text, num_lines) = current_text.get();
        if current_text != self.text.text
        {
            let mut vertices = Vec::with_capacity(text.len() * 4);
            let mut indices = Vec::with_capacity(text.len() * 6);
            let text_data = self.atlas.atlas.text
            (
                text,
                num_lines,
                Align::Center,
                &mut |index| indices.push(index as u16),
                &mut |(s, t, _), (x, y)| vertices.push(GlyphVertex { position: [x, -y], tex_coords: [s, t] })
            );
            self.text.vertices.data(0, &vertices);
            self.text.indices.data(0, &indices);
            self.text.text = current_text;
            self.text.index_count = text_data.index_count;

            gl
                .render_pass(RenderTarget::Texture(&mut self.framebuffer), RenderPassInfo { clear_color: Some((0.0, 0.0, 0.0)), clear_depth: false })
                .pipeline(&self.text.shader, PipelineInfo { depth_test: false, alpha_blend: false, face_cull: true })
                .uniform_f1(&self.text.height_key, 2.0 / num_lines)
                .uniform_texture(&self.text.atlas_key, &self.atlas.texture, false)
                .draw(&self.text.vertices, Some(&self.text.indices), 0, self.text.index_count);
        } 
        //cube
        let mat = 
            Mat4::perspective_opengl(window_dims.0 as f32 / window_dims.1 as f32, std::f32::consts::FRAC_PI_8, 7.0, 10.0)
          * Mat4::translation_z(-9.0)
          * self.rot.to_mat4();
        gl
            .render_pass(RenderTarget::Screen, RenderPassInfo { clear_color: Some((0.2, 0.1, 0.8)), clear_depth: true })
            .pipeline(&self.cube.shader, PipelineInfo { depth_test: true, alpha_blend: false, face_cull: true })
            .uniform_mat4(&self.cube.mat_key, mat)
            .uniform_texture(&self.cube.tex_key, self.framebuffer.texture(), false)
            .draw(&self.cube.vertices, Some(&self.cube.indices), 0, 36);
        true
    }
}
