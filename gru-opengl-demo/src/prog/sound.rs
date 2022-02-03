use std::{path::PathBuf, time::Duration, sync::Arc};

use gru_misc::io::SliceReadSeek;
use gru_opengl::{resource::{id::Id, Load, Loadprotocol, ResSys, ResourceSystem}, impl_ResourceSystem, log, Context};
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source, buffer::SamplesBuffer};

const SOUND_COOLDOWN: f32 = 0.5;
pub struct SoundSystem
{
    pub device: Option<(OutputStream, OutputStreamHandle)>, //on web we need to wait for input -> Option
    pub resources: ResSys<SoundResources>,
    pub cooldown_eh: f32,
    pub cooldown_weh: f32,
}

impl_ResourceSystem!(SoundResources = 
    (eh, SoundData, "eh", ()),
    (weh, SoundData, "weh", ())
);

impl SoundSystem{
    pub fn new(ctx: &mut Context) -> Self 
    {
        Self
        {
            device: None,
            resources: SoundResources::new_loading(2, ctx),
            cooldown_eh: 0.0,
            cooldown_weh: 0.0
        }
    }

    pub fn init_audio(&mut self)
    {
        if self.device.is_none()
        {
            log("init audio");
            self.device = Some(OutputStream::try_default().unwrap());
        }
    }

    pub fn play_eh(&mut self)
    {
        if self.cooldown_eh <= 0.0
        {
            self.cooldown_eh = SOUND_COOLDOWN;
            self.play_audio(self.resources.eh.get());
        }
    }

    pub fn play_weh(&mut self)
    {
        if self.cooldown_weh <= 0.0
        {
            self.cooldown_weh = SOUND_COOLDOWN;
            self.play_audio(self.resources.weh.get());
        }
    }

    fn play_audio(&self, sd: &SoundData)
    {
        if self.resources.finished_loading() 
        {
            if let Some((_, device)) = &self.device 
            {
                device.play_raw(sd.buffer()).unwrap();
            }
        }
        
    }
}

pub struct SoundData 
{
    pub channels: u16,
    pub sample_rate: u32,
    pub data: Vec<f32>,
    duration: Duration,
}

pub struct SoundBuffer
{
    channels: u16,
    sample_rate: u32,
    duration: Duration,
    data: std::slice::Iter<'static, f32>,
}

impl Load for SoundData 
{
    type Config = ();
    fn load(key_gen: &mut Id<u64>, file_path: &std::path::PathBuf, ctx: &mut gru_opengl::Context) -> gru_opengl::resource::Loadprotocol 
    {
        let mut lp = Loadprotocol::empty(format!("Sound {file_path:?}"));
        lp.request_file(key_gen, &file_path.to_string_lossy(), "file", ctx);
        lp
    }

    fn interpret(lp: &gru_opengl::resource::Loadprotocol, _gl: &mut gru_opengl::gl::Gl, _: &mut Self::Config) -> Self 
    {
        let decoder = Decoder::new_vorbis(SliceReadSeek::new(&lp.get_data("file"))).unwrap();
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let data = decoder.convert_samples::<f32>().collect::<Vec<_>>();
        SoundData::new(channels, sample_rate, data)
    }

    fn path(file_name: &'static str) -> std::path::PathBuf 
    {
        PathBuf::from("sounds").join(file_name).with_extension("ogg")
    }
}

impl SoundData {
    fn buffer(&self) -> SoundBuffer
    {
        SoundBuffer{ channels: self.channels, sample_rate: self.sample_rate, duration: self.duration, data: self.data.clone().iter() }
    }

    fn new(channels: u16, sample_rate: u32, data: Vec<f32>) -> Self {
        assert!(channels != 0);
        assert!(sample_rate != 0);

        let duration_ns = 1_000_000_000u64.checked_mul(data.len() as u64).unwrap()
            / sample_rate as u64
            / channels as u64;
        let duration = Duration::new(
            duration_ns / 1_000_000_000,
            (duration_ns % 1_000_000_000) as u32,
        );

        SoundData {
            data: data,
            channels,
            sample_rate,
            duration,
        }
    }
}


impl Source for SoundBuffer {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(self.duration)
    }
}

impl<'a> Iterator for SoundBuffer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().cloned()
    }
}