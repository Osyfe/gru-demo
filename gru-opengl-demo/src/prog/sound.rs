use std::collections::HashMap;

use gru_opengl::resource::{id::Id, Load};
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source, buffer::SamplesBuffer};

pub struct SoundData
{
    device: Option<(OutputStream, OutputStreamHandle)>, //on web we need to wait for input -> Option
    map: HashMap<String, (u16, u32, Vec<f32>)>, //name -> (channels, sample_rate, data)
    cooldown_eh: f32,
    cooldown_weh: f32
}

impl Load for SoundData {
    fn load(key: &mut Id<u64>, path: &std::path::PathBuf, ctx: &mut gru_opengl::Context) -> gru_opengl::resource::Loadprotocol {
        todo!()
    }

    fn interpret(lp: &gru_opengl::resource::Loadprotocol, gl: &mut gru_opengl::gl::Gl) -> Self {
        todo!()
    }

    fn path(name: &'static str) -> std::path::PathBuf {
        todo!()
    }
}