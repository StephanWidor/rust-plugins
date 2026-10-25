use core::ops::RangeInclusive;
use egui::{Align, DragValue, Layout, Response, Ui, Widget, emath::Numeric};
use num::{Integer, rational::Ratio};

pub trait RatioInteger: Integer + Numeric {}
impl<T: Integer + Numeric> RatioInteger for T {}

#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct RatioControl<'a, T: RatioInteger> {
    value: &'a mut Ratio<T>,
    numerator_range: RangeInclusive<T>,
    denominator_range: RangeInclusive<T>,
    speed: f64,
}

impl<'a, T: RatioInteger> RatioControl<'a, T> {
    pub fn new(value: &'a mut Ratio<T>) -> Self {
        assert!(*value.denom() != T::zero());
        let denominator_range = if *value.denom() > T::zero() {
            T::one()..=T::MAX
        } else {
            T::MIN..=(T::zero() - T::one())
        };
        Self {
            value,
            numerator_range: T::MIN..=T::MAX,
            denominator_range: denominator_range,
            speed: 1.0,
        }
    }

    pub fn numerator_range(mut self, range: RangeInclusive<T>) -> Self {
        let numerator_clamped = self.value.numer().clamp(range.start(), range.end());
        *self.value = Ratio::new_raw(*numerator_clamped, *self.value.denom());
        self.numerator_range = range;
        self
    }

    /// `range` should never include 0.
    pub fn denominator_range(mut self, range: RangeInclusive<T>) -> Self {
        assert!(*range.start() * *range.end() > T::zero());
        let denominator_clamped = self.value.denom().clamp(range.start(), range.end());
        *self.value = Ratio::new_raw(*self.value.numer(), *denominator_clamped);
        self.denominator_range = range;
        self
    }

    /// Speed for the `DragValue`s.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }
}

impl<T: RatioInteger> Widget for RatioControl<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut numerator = *self.value.numer();
        let mut denominator = *self.value.denom();

        let numerator_drag = DragValue::new(&mut numerator)
            .speed(self.speed)
            .range(self.numerator_range);
        let denominator_drag = DragValue::new(&mut denominator)
            .speed(self.speed)
            .range(self.denominator_range);

        let response = if ui.layout().prefer_right_to_left() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut response = ui.add(denominator_drag);
                ui.label("/");
                response |= ui.add(numerator_drag);
                response
            })
        } else {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let mut response = ui.add(numerator_drag);
                ui.label("/");
                response |= ui.add(denominator_drag);
                response
            })
        }
        .inner;

        if response.changed() {
            *self.value = Ratio::new_raw(numerator, denominator);
        }
        response
    }
}
