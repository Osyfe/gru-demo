use gru_opengl::{resource::load::Model, gl::{Shader, Texture, TextureConfig, TextureChannel, TextureWrap}, impl_ResourceSystem};
use super::cube::CubeVertex; //, sound::SoundData};

impl_ResourceSystem!(Resources = 
    (cube_model, Model<CubeVertex>, "cube", ()),
    (cube_shader, Shader<CubeVertex>, "cube", ()),
    (cube_texture, Texture<false>, "cube", TextureConfig { size: 0, channel: TextureChannel::RGB, mipmap: true, wrap: TextureWrap::Clamp })
);