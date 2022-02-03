use gru_opengl::{resource::load::{Model, TextureLoadConfig} , gl::{Shader, Texture, TextureChannel, TextureWrap}, impl_ResourceSystem};
use super::cube::CubeVertex; //, sound::SoundData};

impl_ResourceSystem!(Resources = 
    (cube_model, Model<CubeVertex>, "cube", ()),
    (cube_shader, Shader<CubeVertex>, "cube", ()),
    (cube_texture, Texture<false>, "cube", TextureLoadConfig { channel: TextureChannel::RGB, mipmap: true, wrap: TextureWrap::Clamp })
);