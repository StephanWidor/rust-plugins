use super::*;

pub fn param_slider(
    ui: &mut ::egui::Ui,
    param: &FloatParam,
    param_setter: &ParamSetter<'_>,
    modulated_value: Option<f32>,
    prefix: &str,
    suffix: &str,
) -> f32 {
    let mut value = param.value();
    let mut slider =
        egui_utils::custom::Slider::new(&mut value, to_range_inclusive(&param.range()))
            .prefix(prefix)
            .suffix(suffix);
    if let Some(modulated) = modulated_value {
        slider = slider.display_another_value(modulated);
    }
    let response = ui.add(slider);
    if response.changed() {
        set_param(param, value, param_setter);
    }
    value
}

pub fn param_checkbox(
    ui: &mut ::egui::Ui,
    param: &BoolParam,
    param_setter: &ParamSetter<'_>,
    text: &str,
) -> bool {
    let mut value = param.value();
    if ui.checkbox(&mut value, text).clicked() {
        set_param(param, value, param_setter);
    }
    value
}
