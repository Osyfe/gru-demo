use gru_opengl::{log, App, Context, gl::*, event, ui2::Binding as UiBinding, resource::{ResSys, ResourceSystem}};
use gru_misc::{math::*, text_sdf::*};
use gru_ui as ui;

mod cube;
mod sound;
use cube::CubeResources;

use self::sound::SoundSystem;

const TARGET_ROT: Vec3 = Vec3(0.5, 0.5, 0.5);
const ACC: f32 = 0.003;
const WEH_VEL: f32 = 10.0;

struct InputData
{
    last_pos: (f32, f32),
    mouse_down: bool
}

type UiData = String;

pub struct Demo
{
    run_id: u64,
    rot: Rotor,
    vel: Vec3,
    input: InputData,
    sound: SoundSystem,
    ui_data: UiData,
    ui: ui::Ui<'static, UiData>,
    ui_binding: UiBinding,
    cube_resources: ResSys<CubeResources>,
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
        let cube_resources = CubeResources::new_loading(ctx);
        //graphic
        let gl = ctx.gl();
        //ui
        let (ui_data, ui, ui_binding) =
        {
            let ui_data = "Hello gru_ui!".to_owned();
            let ui_binding = UiBinding::new(gl);

            let label = ui::widget::Label::new().size(5.0);

            let font = Font::new(include_bytes!("../res/futuram.ttf"));
            let ui = ui::Ui::new(font, label);

            (ui_data, ui, ui_binding)
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
            sound: SoundSystem::new_loading(ctx),
            ui_data, ui, ui_binding, cube_resources
        }
	}

    fn input(&mut self, ctx: &mut Context, event: event::Event)
    {
        let (width, height) = ctx.window_dims();
        self.ui_binding.event(Vec2(width as f32, height as f32), &event);
        use event::*;
        match event
        {
            Event::File(Ok(file)) => 
                if self.cube_resources.needs_key(&file.key)
                {
                    self.cube_resources.add_file_event(file, ctx.gl());
                } else if self.sound.resources.needs_key(&file.key)
                {
                    self.sound.resources.add_file_event(file, ctx.gl());
                }
            Event::File(Err(err)) => log(err.as_str()),
            Event::Click { button: MouseButton::Left, pressed } =>
            {
                self.input.mouse_down = pressed;
                if self.input.mouse_down { self.sound.play_eh(ctx) }
                else if self.vel.norm() > WEH_VEL { self.sound.play_weh(ctx); }
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
        let size = Vec2(width as f32, height as f32);
        let ui_config = ui::UiConfig
        {
            size,
            scale: 1.0,
            display_scale_factor: 1.0 //ignore here...
        };
        let ui::Frame { paint, .. } = self.ui.frame(ui_config, &mut self.ui_data, self.ui_binding.events().iter());
        self.ui_binding.frame(size, gl, paint);
        
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
        if self.cube_resources.finished_loading() {
            let res = &self.cube_resources;
            let mat = 
                Mat4::perspective_opengl(width as f32 / height as f32, std::f32::consts::FRAC_PI_8, 7.0, 10.0)
            * Mat4::translation_z(-9.0)
            * self.rot.to_mat4();
            rp
                .pipeline(&res.shader.get(), PipelineInfo { depth_test: true, alpha_blend: false, face_cull: true })
                .uniform_name("mat", &mat)
                .uniform_name("tex", res.texture.get())
                .draw(Primitives::Triangles, &res.model.get().vertices, Some(&res.model.get().indices), 0, res.model.get().indices.len() as u32);
        }
        //ui
        self.ui_binding.render(&mut rp);

        true
    }

    fn deinit(&mut self, ctx: &mut Context)
    {
        ctx.set_storage("ID", Some(&format!("{}", self.run_id + 1))); //write storage
    }
}
