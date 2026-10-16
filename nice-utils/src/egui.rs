use super::*;

pub fn param_slider(
    ui: &mut ::egui::Ui,
    param: &FloatParam,
    param_setter: &ParamSetter<'_>,
    prefix: &str,
    suffix: &str,
) -> f32 {
    let mut value = param.value();
    let response = ui.add(
        ::egui::Slider::new(&mut value, to_range_inclusive(&param.range()))
            .prefix(prefix)
            .suffix(suffix),
    );
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
