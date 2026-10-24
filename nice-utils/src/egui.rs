use ::egui::InnerResponse;

use super::*;

pub fn param_slider(
    ui: &mut ::egui::Ui,
    param: &FloatParam,
    param_setter: &ParamSetter<'_>,
    modulated_value: Option<f32>,
    prefix: &str,
    suffix: &str,
) -> ::egui::InnerResponse<f32> {
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
    InnerResponse {
        response: response,
        inner: value,
    }
    //value
}

pub fn param_checkbox(
    ui: &mut ::egui::Ui,
    param: &BoolParam,
    param_setter: &ParamSetter<'_>,
    text: &str,
) -> ::egui::InnerResponse<bool> {
    let mut value = param.value();
    let response = ui.checkbox(&mut value, text);
    if response.clicked() {
        set_param(param, value, param_setter);
    }
    ::egui::InnerResponse {
        response: response,
        inner: value,
    }
}
