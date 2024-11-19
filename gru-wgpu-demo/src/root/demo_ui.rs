use gru_wgpu::ui::{self, Widget, widget::{primitive::{Label, Slider, Check}, layout::{Flex, FlexLayout}, WidgetExt}, lens};
use super::{Demo, light};

pub type UiEvent = ();

pub fn build() -> ui::Ui<'static, Demo, UiEvent>
{
    let font = ui::text::Font::new(include_bytes!("../Latinia.ttf"));

    let ambient_r_slider = Slider::new().min(0.0).max(2.0).lens(lens::LensTuple0).lens(light::Lens_Light_ambient_color);
    let ambient_g_slider = Slider::new().min(0.0).max(2.0).lens(lens::LensTuple1).lens(light::Lens_Light_ambient_color);
    let ambient_b_slider = Slider::new().min(0.0).max(2.0).lens(lens::LensTuple2).lens(light::Lens_Light_ambient_color);
    let sun_r_slider = Slider::new().min(0.0).max(10.0).lens(lens::LensTuple0).lens(light::Lens_Light_sun_color);
    let sun_g_slider = Slider::new().min(0.0).max(10.0).lens(lens::LensTuple1).lens(light::Lens_Light_sun_color);
    let sun_b_slider = Slider::new().min(0.0).max(10.0).lens(lens::LensTuple2).lens(light::Lens_Light_sun_color);
    let rot_slider = Slider::new().min(-3.0).max(3.0).lens(light::Lens_Light_rot_speed);
    let theta_slider = Slider::new().min(0.1).max(std::f32::consts::PI - 0.1).lens(light::Lens_Light_theta);

    fn show(name: &'static str, mode: light::Present) -> impl Widget<light::Light, UiEvent>
    {
        let show = Check::new()
            .lens(lens::Transform::new(move |present| *present == mode, move |present, set| if *set { *present = mode }))
            .lens(light::Lens_Light_present)
            .response();
        Flex::row().layout(FlexLayout::PadBetween).with(Label::new().own(name)).with(show)
    }
    let show_full = show("Full", light::Present::Full);
    let show_normal = show("Normal", light::Present::Normal);
    let show_roughness = show("Roughness", light::Present::Roughness);

    let light = Flex::column()
        .with(Label::new().own("Ambient Color:"))
        .with(ambient_r_slider)
        .with(ambient_g_slider)
        .with(ambient_b_slider)
        .with(Label::new().own("Sun Color:"))
        .with(sun_r_slider)
        .with(sun_g_slider)
        .with(sun_b_slider)
        .with(Label::new().own("Sun Rotation Speed:"))
        .with(rot_slider)
        .with(Label::new().own("Sun Theta:"))
        .with(theta_slider)
        .with(Label::new().own("Show:"))
        .with(show_full)
        .with(show_normal)
        .with(show_roughness)
        .fix().width(10.0)
        .pad().all(1.0)
        .lens(super::Lens_Demo_light);

    ui::Ui::new(font, light)
}
