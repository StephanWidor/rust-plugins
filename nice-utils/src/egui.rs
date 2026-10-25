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

pub fn param_combo_box<ENUM: Enum + PartialEq + Copy + 'static>(
    ui: &mut ::egui::Ui,
    width: f32,
    param: &EnumParam<ENUM>,
    param_setter: &ParamSetter<'_>,
) -> ENUM {
    let mut selection = param.value();
    let mut selection_has_changed = false;
    ::egui::ComboBox::from_id_salt(ui.next_auto_id())
        .selected_text(param.to_string())
        .width(width)
        .show_ui(ui, |ui| {
            let variant_strings = &ENUM::variants();
            for i in 0..variant_strings.len() {
                let response =
                    ui.selectable_value(&mut selection, Enum::from_index(i), variant_strings[i]);
                selection_has_changed |= response.changed();
            }
        });
    if selection_has_changed {
        set_param(param, selection, param_setter);
    }
    selection
}

pub fn param_ratio_control(
    ui: &mut ::egui::Ui,
    numerator: &IntParam,
    denominator: &IntParam,
    param_setter: &ParamSetter<'_>,
    speed: f64,
) -> ::egui::InnerResponse<num::rational::Ratio<i32>> {
    // We use `Ratio::new_raw` because `Ratio::new` reduces values!
    let mut ratio = num::rational::Ratio::new_raw(numerator.value(), denominator.value());
    let response = ui.add(
        egui_utils::custom::RatioControl::new(&mut ratio)
            .speed(speed)
            .numerator_range(to_range_inclusive_int(&numerator.range()))
            .denominator_range(to_range_inclusive_int(&denominator.range())),
    );
    if response.changed() {
        set_params(
            [numerator, denominator],
            [*ratio.numer(), *ratio.denom()],
            param_setter,
        );
    }
    ::egui::InnerResponse {
        response: response,
        inner: ratio,
    }
}
