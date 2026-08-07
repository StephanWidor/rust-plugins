use nalgebra::*;

use super::*;

pub const NUM_CONTROL_POINTS: usize = 4;

#[derive(nice::Params)]
pub struct Point {
    #[id = "x"]
    pub x: nice::FloatParam,

    #[id = "y"]
    pub y: nice::FloatParam,
}

impl Point {
    pub fn new(x: f32, y: f32, index: usize) -> Self {
        Self {
            x: nice::FloatParam::new(
                format!("Path x [{}]", index + 1).as_str(),
                x,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            y: nice::FloatParam::new(
                format!("Path y [{}]", index + 1).as_str(),
                y,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
        }
    }

    pub fn get<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        [F::from_float(self.x.value()), F::from_float(self.y.value())]
    }

    pub fn set(&self, x: f32, y: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.x);
        setter.begin_set_parameter(&self.y);
        setter.set_parameter(&self.x, x);
        setter.set_parameter(&self.y, y);
        setter.end_set_parameter(&self.x);
        setter.end_set_parameter(&self.y);
    }
}

#[derive(nice::Params)]
pub struct Path {
    #[id = "Enabled"]
    pub enabled: nice::BoolParam,

    #[id = "t"]
    pub t: nice::FloatParam,

    #[nested(array, group = "control_points")]
    pub control_points: [Point; NUM_CONTROL_POINTS],

    spline_base: Matrix4<f32>,
}

impl Path {
    pub fn new() -> Self {
        Self {
            enabled: nice::BoolParam::new("Path Enabled", false),
            t: nice::FloatParam::new(
                "Path t",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            ),
            control_points: [
                Point::new(0.1_f32, 0.6_f32, 0),
                Point::new(0.2_f32, 0_f32, 1),
                Point::new(0.5_f32, 0.8_f32, 2),
                Point::new(1_f32, 0.6_f32, 3),
            ],
            spline_base: (1_f32 / 3_f32)
                * nalgebra::matrix![
                    3_f32,   0_f32,   0_f32,   0_f32;
                    -19_f32, 24_f32,  -8_f32,  3_f32;
                    32_f32,  -56_f32, 40_f32,  -16_f32;
                    -16_f32, 32_f32,  -32_f32, 16_f32],
        }
    }

    pub fn set_enabled(&self, enabled: bool, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.enabled);
        setter.set_parameter(&self.enabled, enabled);
        setter.end_set_parameter(&self.enabled);
    }

    pub fn set_t(&self, t: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.t);
        setter.set_parameter(&self.t, t);
        setter.end_set_parameter(&self.t);
    }

    pub fn get_point<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        Self::point_from_coefficients_and_t(&self.coefficients_matrix(), self.t.value())
    }

    /// coefficients[0] + coefficients[1] * t + coefficients[2] * t^2 + ... + coefficients[n-1] * t^(n-1)
    /// (calculated via horner scheme)
    pub fn calc_horner(coefficients: &[f32], t: f32) -> f32 {
        let n = coefficients.len();
        let mut accumulator = coefficients[n - 1];
        for j in 2..=n {
            accumulator = accumulator * t + coefficients[n - j];
        }
        accumulator
    }

    pub fn point_from_coefficients_and_t<F: audio_lib::utils::Float>(
        coefficients: &SMatrix<f32, NUM_CONTROL_POINTS, 2>,
        t: f32,
    ) -> [F; 2] {
        [
            F::from_float(
                Self::calc_horner(coefficients.column(0).as_slice(), t).clamp(0_f32, 1_f32),
            ),
            F::from_float(
                Self::calc_horner(coefficients.column(1).as_slice(), t).clamp(0_f32, 1_f32),
            ),
        ]
    }

    pub fn coefficients_matrix(&self) -> SMatrix<f32, NUM_CONTROL_POINTS, 2> {
        self.spline_base * self.geometry_matrix()
    }

    fn geometry_matrix(&self) -> SMatrix<f32, NUM_CONTROL_POINTS, 2> {
        SMatrix::from_fn(|row, col| {
            let control_point = &self.control_points[row];
            if col == 0 {
                control_point.x.value()
            } else {
                control_point.y.value()
            }
        })
    }
}
