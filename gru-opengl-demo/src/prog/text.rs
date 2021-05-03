use gru_opengl::{AttributesReprCpacked, BufferType};

pub const ATLAS_SIZE: u32 = 1024;

pub const TEXTS: [&'static str; 1] =
[
    "Hello Open GL"
];

#[derive(PartialEq)]
pub enum Text
{
    None,
    Hello
}

impl Text
{
    pub fn get(&self) -> (&'static str, f32)
    {
        match self
        {
            Self::None => unreachable!(),
            Self::Hello => (TEXTS[0], 3.0)
        }
    }
}

#[repr(C, packed)]
pub struct GlyphVertex
{
    pub position: [f32; 2],
    pub tex_coords: [f32; 2]
}

impl AttributesReprCpacked for GlyphVertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 2 }, "position"),
        (BufferType::Float { size: 2 }, "tex_coords")
    ];
}
