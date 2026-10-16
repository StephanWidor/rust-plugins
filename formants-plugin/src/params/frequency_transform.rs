use audio_lib::utils;
use std::ops::RangeInclusive;

/// For transforming/distorting coordinates (x,y) in [0;1]^2 into formant frequencies
/// (see doc folder for explanation)
pub struct FrequencyTransform {
    pub bounds_matrix: nalgebra::Matrix2x4<f32>,
    pub frequency_ranges: [RangeInclusive<f32>; 2],
    m0: [f32; 2],
    a: [f32; 2],
    b: [f32; 2],
    c: [f32; 2],
    u: f32,
    v: f32,
}

impl Default for FrequencyTransform {
    fn default() -> Self {
        let bounds_matrix: nalgebra::Matrix2x4<f32> = nalgebra::matrix![
            300_f32, 1000_f32, 1000_f32,  260_f32;
            800_f32, 1400_f32, 2000_f32, 3200_f32];
        let f_ranges: [RangeInclusive<f32>; 2] =
            std::array::from_fn(|i| bounds_matrix.row(i).min()..=bounds_matrix.row(i).max());
        let a = std::array::from_fn(|i| bounds_matrix.row(i)[0] - bounds_matrix.row(i)[1]);
        let b = std::array::from_fn(|i| bounds_matrix.row(i)[0] - bounds_matrix.row(i)[3]);
        let c = std::array::from_fn(|i| {
            bounds_matrix.row(i)[0] - bounds_matrix.row(i)[1] + bounds_matrix.row(i)[2]
                - bounds_matrix.row(i)[3]
        });
        let u = a[1] * c[0] - a[0] * c[1];
        let v = b[0] * c[1] - b[1] * c[0];
        Self {
            bounds_matrix: bounds_matrix,
            frequency_ranges: f_ranges,
            m0: std::array::from_fn(|i| bounds_matrix.row(i)[0]),
            a: a,
            b: b,
            c: c,
            u: u,
            v: v,
        }
    }
}

impl FrequencyTransform {
    pub fn coordinates_to_frequency(&self, x: f32, y: f32, index: usize) -> f32 {
        assert!(index < 2);
        self.m0[index] - self.a[index] * x - self.b[index] * y + self.c[index] * x * y
    }

    pub fn coordinates_to_frequencies(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.coordinates_to_frequency(x, y, 0),
            self.coordinates_to_frequency(x, y, 1),
        ]
    }

    pub fn gain_db_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        let ratio = self.frequency_ratio(frequency, index);
        if index == 0 {
            12_f32 - ratio * 6_f32
        } else {
            6_f32 + ratio * 6_f32
        }
    }

    pub fn q_for_frequency(&self, frequency: f32, index: usize) -> f32 {
        if index == 0 {
            10_f32
        } else {
            8_f32 + self.frequency_ratio(frequency, index) * 10_f32
        }
    }

    pub fn xy_from_frequencies(&self, frequencies: [f32; 2]) -> Option<[f32; 2]> {
        if self.v == 0_f32 {
            return None;
        }
        let w =
            self.c[0] * (frequencies[1] - self.m0[1]) - self.c[1] * (frequencies[0] - self.m0[0]);
        let c_2 = self.c[0] * self.u / self.v;
        let c_1 = (self.c[0] * w - self.b[0] * self.u) / self.v - self.a[0];
        let c_0 = self.m0[0] - self.b[0] * w / self.v - frequencies[0];
        let polynom_roots = utils::polynom_roots(c_2, c_1, c_0);
        const EPS: f32 = 1e-4_f32;
        for root in polynom_roots {
            if root.re >= 0_f32 - EPS && root.re <= 1_f32 + EPS && root.im.abs() <= EPS {
                let x = root.re.clamp(0_f32, 1_f32);
                let y = (self.u * x / self.v + w / self.v).clamp(0_f32, 1_f32);
                return Some([x, y]);
            }
        }
        None
    }

    fn frequency_ratio(&self, frequency: f32, index: usize) -> f32 {
        assert!(index < 2);
        ((frequency - self.frequency_ranges[index].start())
            / utils::get_length(&self.frequency_ranges[index]))
        .clamp(0_f32, 1_f32)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_bounds() {
        let bound_coords = [
            [0_f32, 0_f32],
            [1_f32, 0_f32],
            [1_f32, 1_f32],
            [0_f32, 1_f32],
        ];
        let transform = FrequencyTransform::default();
        for i in 0..4 {
            let [x, y] = bound_coords[i];
            let bound = transform.bounds_matrix.column(i);
            let frequencies = transform.coordinates_to_frequencies(x, y);
            for j in 0..2 {
                assert_approx_eq!(bound[j], frequencies[j]);
            }
        }
    }

    #[test]
    fn test_xy_from_frequencies() {
        let t = FrequencyTransform::default();
        const EPS: f32 = 1e-4_f32;
        for i in 0..=10 {
            for j in 0..=10 {
                let [x, y] = [i as f32 * 0.1, j as f32 * 0.1];
                let frequencies = t.coordinates_to_frequencies(x, y);
                let xy_back = t.xy_from_frequencies(frequencies);
                assert!(xy_back.is_some());
                let [x_back, y_back] = xy_back.unwrap();
                let diff_x = (x - x_back).abs();
                assert!(diff_x <= EPS);
                let diff_y = (y - y_back).abs();
                assert!(diff_y <= EPS);
            }
        }
    }
}
