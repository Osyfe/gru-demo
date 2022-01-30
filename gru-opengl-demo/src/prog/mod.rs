use std::collections::{HashSet, HashMap};
use gru_opengl::{log, App, Context, gl::*, event, ui};
use gru_misc::{math::*, text::*, io::*};
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source, buffer::SamplesBuffer};

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
    texture: Texture<false>
}

struct TextData
{
    shader: Shader<GlyphVertex>,
    vertices: VertexBuffer<GlyphVertex>,
    indices: IndexBuffer,
    text: Text,
    index_count: u32,
    height_key: UniformKey<f32>,
    atlas_key: UniformKey<Texture<false>>
}

struct CubeData
{
    shader: Shader<CubeVertex>,
    vertices: VertexBuffer<CubeVertex>,
    indices: IndexBuffer,
    mat_key: UniformKey<Mat4>,
    tex_key: UniformKey<Texture<true>>
}

struct InputData
{
    last_pos: (f32, f32),
    mouse_down: bool
}

struct SoundData
{
    device: Option<(OutputStream, OutputStreamHandle)>, //on web we need to wait for input -> Option
    map: HashMap<String, (u16, u32, Vec<f32>)>, //name -> (channels, sample_rate, data)
    cooldown_eh: f32,
    cooldown_weh: f32
}

struct UiData
{
    size: Vec2
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
    sound: SoundData,
    ui_data: UiData,
    ui: ui::Ui<'static, UiData>,
    ui_events: Vec<ui::event::Event>,
    ui_binding: ui::Binding
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
    type Init = ();

	fn init(ctx: &mut Context, _: Self::Init) -> Self
	{
        gru_opengl::log("init app");
        ctx.set_title("gru_opengl_demo");
        //read storage
        let run_id = match ctx.get_storage("ID")
        {
            Some(id) => id.parse().unwrap(),
            None => 0
        };
        log(&format!("run_id = {}", run_id));
        //load files
        ctx.load_file("eh.ogg", 0);
        ctx.load_file("weh.ogg", 0);
        //graphic
        let gl = ctx.gl();
        //font
        let (atlas, atlas_texture) =
        {
            let mut alphabet = HashSet::new();
            for text in &TEXTS { for c in text.chars() { if c != ' ' { alphabet.insert(c); } } }
            let font = Font::new(include_bytes!("../res/futuram.ttf"));
            let (font_texture, atlas) = Atlas::new(font, alphabet, 100.0, ATLAS_SIZE, 5);
            if font_texture.len() > 1 { panic!("Atlas more than one page!"); }
            let atlas_texture = gl.new_texture(&TextureConfig { size: ATLAS_SIZE, channel: TextureChannel::A, mipmap: false, wrap: TextureWrap::Repeat }, &font_texture[0]);
            (atlas, atlas_texture)
        };
        //text
        let max_chars = TEXTS.iter().map(|text| text.chars().count() as u32).max().unwrap();
        let text_shader = gl.new_shader(include_str!("../glsl/text.vert"), include_str!("../glsl/text.frag"));
        let text_vertices = gl.new_vertex_buffer(max_chars * 4, BufferAccess::Dynamic);
        let text_indices = gl.new_index_buffer(max_chars * 6, BufferAccess::Dynamic);
        let height_key = text_shader.get_key("height");
        let atlas_key = text_shader.get_key("atlas");
        //cube
        let cube_shader = gl.new_shader(include_str!("../glsl/cube.vert"), include_str!("../glsl/cube.frag"));
        let (indices, vertices) = cube();
        let mut cube_vertices = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::Static);
        cube_vertices.data(0, &vertices);
        let mut cube_indices = gl.new_index_buffer(indices.len() as u32, BufferAccess::Static);
        cube_indices.data(0, &indices);   
        let mat_key = cube_shader.get_key("mat");
        let tex_key = cube_shader.get_key("tex");
        //framebuffer
        let framebuffer = gl.new_framebuffer(&FramebufferConfig { depth: false, size: FRAMEBUFFER_SIZE, wrap: TextureWrap::Clamp });
        //ui
        let (ui_data, ui, ui_events, ui_binding) =
        {
            let ui_data = UiData { size: Vec2(1.0, 1.0) };
            let ui_events = Vec::new();
            let ui_binding = ui::Binding::new(gl);
            let font = Font::new(include_bytes!("../res/futuram.ttf"));
            let mut ui = ui::Ui::new(font, |data: &UiData| ui::UiConfig { size: data.size, scale: 1.0, display_scale_factor: 1.0 }); //ignore display scale
            
            use ui::{widget::{WidgetExt, Square, Label}, layout::{LayoutAlign, Flex, Split}};
            use gru_misc::{paint::TextSize};
            let column = Flex::column(0.5, LayoutAlign::Front, LayoutAlign::Fill)
                .with(Label::new("Small", TextSize::Small, Align::Right))
                .with(Square::new().response(Some(Box::new(|| println!("Button 1")))))
                .with(Label::new("Normal", TextSize::Normal, Align::Center))
                .with(Square::new())
                .with(Label::new("Large", TextSize::Large, Align::Left))
                .align(LayoutAlign::Fill, LayoutAlign::Front)
                .padding(Vec2(1.0, 1.0));
            ui.add(Split::row([column.boxed(), Square::new().boxed()], None));

            (ui_data, ui, ui_events, ui_binding)
        };
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
            },
            ui_data, ui, ui_events, ui_binding
        }
	}

    fn input(&mut self, ctx: &mut Context, event: event::Event)
    {
        if let Some(event) = self.ui_binding.event(self.ui_data.size, &event) { self.ui_events.push(event); }
        use event::*;
        match event
        {
            Event::File(Ok(File { path: name, key: _, data })) =>
            {
                let decoder = Decoder::new_vorbis(SliceReadSeek::new(&data)).unwrap();
                let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
                let data = decoder.convert_samples::<f32>().collect::<Vec<_>>();
                self.sound.map.insert(name, (channels, sample_rate, data));
            },
            Event::File(Err(err)) => log(err.as_str()),
            Event::Click { button: MouseButton::Left, pressed } =>
            {
                self.init_audio();
                self.input.mouse_down = pressed;
                if self.input.mouse_down { self.play_eh() }
                else if self.vel.norm() > WEH_VEL { self.play_weh(); }
            },
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
            Event::Key { key: KeyCode::Space, pressed: true } =>
            {
                ctx.set_fullscreen(!ctx.fullscreen());
            },
            _ => {}
        }
    }

    fn frame(&mut self, ctx: &mut Context, dt: f32) -> bool
    {
        let (width, height) = ctx.window_dims();
        let gl = ctx.gl();
        //ui
        self.ui_data.size = Vec2(width as f32, height as f32);
        let ui::Frame { paint, .. } = self.ui.frame(&mut self.ui_data, self.ui_events.iter());
        self.ui_events.clear();
        self.ui_binding.frame(self.ui_data.size, gl, paint);
        //cooldown
        self.sound.cooldown_eh -= dt;
        self.sound.cooldown_weh -= dt;
        //physik
        self.vel += (TARGET_ROT - self.vel) * dt;
        self.rot = Rotor::from_axis(self.vel * dt) * self.rot;
        self.rot.fix();
        //graphic
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
                Layout { width: num_lines, align: Align::Center, auto_wrap: true },
                &mut |index| indices.push(index as u16),
                &mut |(s, t, _), (x, y): (f32, f32)| vertices.push(GlyphVertex { position: [x, -y], tex_coords: [s, t] })
            );
            self.text.vertices.data(0, &vertices);
            self.text.indices.data(0, &indices);
            self.text.text = current_text;
            self.text.index_count = text_data.index_count;

            gl
                .render_pass(RenderTarget::Texture(&mut self.framebuffer), RenderPassInfo { clear_color: Some((0.0, 0.0, 0.0)), clear_depth: false })
                .pipeline(&self.text.shader, PipelineInfo { depth_test: false, alpha_blend: false, face_cull: true })
                .uniform_key(&self.text.height_key, &(2.0 / num_lines))
                .uniform_key(&self.text.atlas_key, &self.atlas.texture)
                .draw(Primitives::Triangles, &self.text.vertices, Some(&self.text.indices), 0, self.text.index_count);
        }
        let mut rp = gl.render_pass(RenderTarget::Screen, RenderPassInfo { clear_color: Some((0.2, 0.1, 0.8)), clear_depth: true });
        //cube
        let mat = 
            Mat4::perspective_opengl(width as f32 / height as f32, std::f32::consts::FRAC_PI_8, 7.0, 10.0)
          * Mat4::translation_z(-9.0)
          * self.rot.to_mat4();
        rp
            .pipeline(&self.cube.shader, PipelineInfo { depth_test: true, alpha_blend: false, face_cull: true })
            .uniform_key(&self.cube.mat_key, &mat)
            .uniform_key(&self.cube.tex_key, self.framebuffer.texture())
            .draw(Primitives::Triangles, &self.cube.vertices, Some(&self.cube.indices), 0, 36);
        //ui
        self.ui_binding.render(&mut rp);

        true
    }

    fn deinit(self, ctx: &mut Context)
    {
        ctx.set_storage("ID", Some(&format!("{}", self.run_id + 1))); //write storage
    }
}
