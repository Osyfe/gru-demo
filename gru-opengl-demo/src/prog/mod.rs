use std::collections::HashMap;
use gru_opengl::{log, App, Context, gl::*, event, ui, resource::{ResSys, ResourceSystem}};
use gru_misc::{math::*, text::*, io::*};
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source, buffer::SamplesBuffer};

mod cube;
mod resources;
//mod sound;
use resources::Resources;

//use self::sound::SoundData;

const TARGET_ROT: Vec3 = Vec3(0.5, 0.5, 0.5);
const ACC: f32 = 0.003;
const WEH_VEL: f32 = 10.0;
const SOUND_COOLDOWN: f32 = 0.5;

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
    rot: Rotor,
    vel: Vec3,
    input: InputData,
    sound: SoundData,
    ui_data: UiData,
    ui: ui::Ui<'static, UiData>,
    ui_events: Vec<ui::event::Event>,
    ui_binding: ui::Binding,
    resources: ResSys<Resources>,
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
        let resources = Resources::new_loading(1, ctx);
        ctx.load_file("eh.ogg", 0);
        ctx.load_file("weh.ogg", 0);
        //graphic
        let gl = ctx.gl();
        //ui
        let (ui_data, ui, ui_events, ui_binding) =
        {
            let ui_data = UiData { size: Vec2(1.0, 1.0) };
            let ui_events = Vec::new();
            let ui_binding = ui::Binding::new(gl);
            let font = Font::new(include_bytes!("../res/futuram.ttf"));
            let mut ui = ui::Ui::new(font, |data: &UiData| ui::UiConfig { size: data.size, scale: 1.0, display_scale_factor: 1.0 }); //ignore display scale
            
            use ui::{widget::{WidgetExt, Label}, layout::{LayoutAlign, Flex, Split}};
            use gru_misc::{paint::TextSize};
            let column = Flex::column(0.5, LayoutAlign::Front, LayoutAlign::Fill)
                .with(Label::new("Small", TextSize::Small, Align::Right).bg().response(Some(Box::new(|| println!("Button 1")))))
                .with(Label::new("Normal", TextSize::Normal, Align::Center))
                .with(Label::new("Large", TextSize::Large, Align::Left))
                .align(LayoutAlign::Fill, LayoutAlign::Front)
                .padding(Vec2(3.0, 1.0));
            ui.add(Split::row([column.boxed(), Label::new("Right side", TextSize::Normal, Align::Center).boxed()], None));

            (ui_data, ui, ui_events, ui_binding)
        };
        //pack everything
		Self
        {
            run_id,
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
            ui_data, ui, ui_events, ui_binding, resources
        }
	}

    fn input(&mut self, ctx: &mut Context, event: event::Event)
    {
        if let Some(event) = self.ui_binding.event(self.ui_data.size, &event) { self.ui_events.push(event); }
        use event::*;
        match event
        {
            Event::File(Ok(File { path: name, key, data })) => if key < 1000
            {
                let decoder = Decoder::new_vorbis(SliceReadSeek::new(&data)).unwrap();
                let (channels, sample_rate) = (decoder.channels(), decoder.sample_rate());
                let data = decoder.convert_samples::<f32>().collect::<Vec<_>>();
                self.sound.map.insert(name, (channels, sample_rate, data));
            } else {
                self.resources.add_file_event(File { path: name, key, data }, ctx.gl());
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
        //cube
        
        let mut rp = gl.render_pass(RenderTarget::Screen, RenderPassInfo { clear_color: Some((0.2, 0.1, 0.8)), clear_depth: true });
        if self.resources.finished_loading() {
            let mat = 
                Mat4::perspective_opengl(width as f32 / height as f32, std::f32::consts::FRAC_PI_8, 7.0, 10.0)
            * Mat4::translation_z(-9.0)
            * self.rot.to_mat4();
            rp
                .pipeline(&self.resources.cube_shader.get(), PipelineInfo { depth_test: true, alpha_blend: false, face_cull: true })
                .uniform_name("mat", &mat)
                .uniform_name("tex", self.resources.cube_texture.get())
                .draw(Primitives::Triangles, &self.resources.cube_model.get().vertices, Some(&self.resources.cube_model.get().indices), 0, self.resources.cube_model.get().indices.len() as u32);
        }
        //ui
        self.ui_binding.render(&mut rp);

        true
    }

    fn deinit(self, ctx: &mut Context)
    {
        ctx.set_storage("ID", Some(&format!("{}", self.run_id + 1))); //write storage
    }
}
