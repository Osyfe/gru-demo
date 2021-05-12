use std::collections::{HashSet, HashMap};
use gru_opengl::{log, App, Context, gl::*, event};
use gru_math::*;
use gru_text::*;
use gru_util::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::{Source}, buffer::SamplesBuffer};

mod text;
mod cube;
use text::*;
use cube::*;

const FRAMEBUFFER_SIZE: u32 = 1024;
const TARGET_ROT: Vec3 = Vec3(0.5, 0.5, 0.5);
const ACC: f32 = 0.003;
const WEH_VEL: f32 = 10.0;
const SOUND_COOLDOWN: f32 = 0.5;

struct AtlasData
{
    atlas: Atlas,
    texture: Texture
}

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

struct InputData
{
    last_pos: (f32, f32),
    mouse_down: bool
}

struct SoundData
{
    device: Option<(OutputStream, OutputStreamHandle)>, //on web we need to wait for input
    map: HashMap<String, (u16, u32, Vec<f32>)>, //name -> (channels, sample_rate, data)
    cooldown_eh: f32,
    cooldown_weh: f32
}

pub struct Demo
{
    run_id: u64,
    atlas: AtlasData,
    text: TextData,
    cube: CubeData,
    framebuffer: Framebuffer,
    rot: Rotor,
    vel: Vec3,
    input: InputData,
    sound: SoundData
}

impl Demo
{
    fn init_audio(&mut self)
    {
        if self.sound.device.is_none()
        {
            log("init audio");
            self.sound.device = Some(OutputStream::try_default().unwrap());
        }
    }

    fn play_audio(&mut self, name: &str)
    {
        if let Some((channels, sample_rate, data)) = self.sound.map.get(name)
        {
            if let Some(device) = &self.sound.device
            {
                let buffer = SamplesBuffer::new(*channels, *sample_rate, data.clone());
                device.1.play_raw(buffer).unwrap();
            }
        }
    }

    fn play_eh(&mut self)
    {
        if self.sound.cooldown_eh <= 0.0
        {
            self.sound.cooldown_eh = SOUND_COOLDOWN;
            self.play_audio("eh.ogg");
        }
    }

    fn play_weh(&mut self)
    {
        if self.sound.cooldown_weh <= 0.0
        {
            self.sound.cooldown_weh = SOUND_COOLDOWN;
            self.play_audio("weh.ogg");
        }
    }
}

impl App for Demo
{
	fn init(ctx: &mut Context) -> Self
	{
        //read storage
        let run_id = match ctx.storage.get("ID")
        {
            Some(id) => id.parse().unwrap(),
            None => 0
        };
        log(&format!("run_id = {}", run_id));
        //load files
        ctx.load_file("eh.ogg");
        ctx.load_file("weh.ogg");
        //graphic
        let gl = &mut ctx.gl;
        //font
        let mut alphabet = HashSet::new();
        for text in &TEXTS { for c in text.chars() { if c != ' ' { alphabet.insert(c); } } }
        let font = Font::new(include_bytes!("../res/futuram.ttf"));
        let (font_texture, atlas) = Atlas::new(&font, &alphabet, 100.0, ATLAS_SIZE, 5);
        if font_texture.len() > 1 { panic!("Atlas more than one page"); }
        let atlas_texture = gl.new_texture(&TextureConfig { size: ATLAS_SIZE, channel: TextureChannel::A, mipmap: false, wrap: TextureWrap::Repeat}, &font_texture[0]);
        //text
        let max_chars = TEXTS.iter().map(|text| text.chars().count() as u32).max().unwrap();
        let text_shader = gl.new_shader(include_str!("../glsl/text.vert"), include_str!("../glsl/text.frag"));
        let text_vertices = gl.new_vertex_buffer(max_chars * 4, BufferAccess::DYNAMIC);
        let text_indices = gl.new_index_buffer(max_chars * 6, BufferAccess::DYNAMIC);
        let height_key = text_shader.get_key("height").unwrap().clone();
        let atlas_key = text_shader.get_key("atlas").unwrap().clone();
        //cube
        let cube_shader = gl.new_shader(include_str!("../glsl/cube.vert"), include_str!("../glsl/cube.frag"));
        let (indices, vertices) = cube();
        let mut cube_vertices = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::STATIC);
        cube_vertices.data(0, &vertices);
        let mut cube_indices = gl.new_index_buffer(indices.len() as u32, BufferAccess::STATIC);
        cube_indices.data(0, &indices);   
        let mat_key = cube_shader.get_key("mat").unwrap().clone();
        let tex_key = cube_shader.get_key("tex").unwrap().clone();
        //framebuffer
        let framebuffer = gl.new_framebuffer(&FramebufferConfig { depth: false, size: FRAMEBUFFER_SIZE, wrap: TextureWrap::Clamp });
        //pack everything
		Self
        {
            run_id,
            atlas: AtlasData
            {
                atlas,
                texture: atlas_texture
            },
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
            framebuffer,
            rot: Rotor::identity(),
            vel: TARGET_ROT,
            input: InputData
            {
                last_pos: (0.0, 0.0),
                mouse_down: false
            },
            sound: SoundData
            {
                device: None,
                map: HashMap::new(),
                cooldown_eh: 0.0,
                cooldown_weh: 0.0
            }
        }
	}

    fn input(&mut self, _ctx: &mut Context, event: event::Event)
    {
        use event::*;
        match event
        {
            Event::File(name, data) =>
            {
                let decoder = Decoder::new_vorbis(SliceReadSeek::new(&data)).unwrap();
                let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
                let data = decoder.convert_samples::<f32>().collect::<Vec<_>>();
                self.sound.map.insert(name, (channels, sample_rate, data));
            },
            Event::Click { button: MouseButton::Left, state } =>
            {
                self.init_audio();
                self.input.mouse_down = state == ElementState::Pressed;
                if self.input.mouse_down { self.play_eh() }
                else if self.vel.norm() > WEH_VEL { self.play_weh(); }
            }
            Event::Cursor { position } =>
            {
                let (x, y) = position;
                let (x2, y2) = self.input.last_pos;
                if self.input.mouse_down
                {
                    let diff = Vec3(y2 - y, x - x2, 0.0);
                    let vel = ACC * diff.norm().sqrt() + ACC;
                    self.vel += diff * vel;
                }
                self.input.last_pos = position;
            },
            Event::Touch { position, phase, .. } =>
            {
                let (x, y) = position;
                match phase
                {
                    TouchPhase::Started =>
                    {
                        self.init_audio();
                        self.play_eh();
                    },
                    TouchPhase::Ended => if self.vel.norm() > WEH_VEL { self.play_weh() },
                    TouchPhase::Moved =>
                    {
                        let (x2, y2) = self.input.last_pos;
                        let diff = Vec3(y2 - y, x - x2, 0.0);
                        let vel = ACC * diff.norm().sqrt() + ACC;
                        self.vel += diff * vel;
                    },
                    TouchPhase::Cancelled => {}
                }
                self.input.last_pos = position;
            },
            _ => {}
        }
    }

    fn frame(&mut self, ctx: &mut Context, dt: f32, window_dims: (u32, u32)) -> bool
    {
        //cooldown
        self.sound.cooldown_eh -= dt;
        self.sound.cooldown_weh -= dt;
        //physi
        self.vel += (TARGET_ROT - self.vel) * dt;
        self.rot = Rotor::from_axis(self.vel * dt) * self.rot;
        self.rot.fix();
        //graphic
        let gl = &mut ctx.gl;
        //text
        let current_text = if self.vel.norm() > WEH_VEL { Text::Wuhu } else { Text::Hello };
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

    fn deinit(self, ctx: &mut Context)
    {
        ctx.storage.set("ID", Some(&format!("{}", self.run_id + 1))); //write storage
    }
}
