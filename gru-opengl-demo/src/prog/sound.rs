use gru_opengl::{resource::{Res, ResSys, ResourceSystem, load::Audio}, impl_ResourceSystem, Context};

const SOUND_COOLDOWN: f32 = 0.5;

pub struct SoundSystem
{
    pub resources: ResSys<SoundResources>,
    pub cooldown_eh: f32,
    pub cooldown_weh: f32,
}

impl_ResourceSystem!(SoundResources = 
    (eh, Audio, "eh", ()),
    (weh, Audio, "weh", ())
);

impl SoundSystem{
    pub fn new_loading( ctx: &mut Context) -> Self 
    {
        Self
        {
            resources: SoundResources::new_loading(ctx),
            cooldown_eh: 0.0,
            cooldown_weh: 0.0
        }
    }

    pub fn play_eh(&mut self, ctx: &Context)
    {
        if self.cooldown_eh <= 0.0
        {
            self.cooldown_eh = SOUND_COOLDOWN;
            self.play_audio(&self.resources.eh, ctx);
        }
    }

    pub fn play_weh(&mut self, ctx: &Context)
    {
        if self.cooldown_weh <= 0.0
        {
            self.cooldown_weh = SOUND_COOLDOWN;
            self.play_audio(&self.resources.weh, ctx);
        }
    }

    fn play_audio(&self, audio: &Res<Audio>, ctx: &Context)
    {
        if let Some(device) = ctx.audio()
        {
            if audio.is_loaded() {
                device.play_raw(audio.get().buffer()).unwrap();
            }
        }
    }
}
