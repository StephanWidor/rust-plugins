use super::*;

pub struct State {
    pub vowels_map: Vec<(String, Option<[f32; 2]>)>,
    pub hovered_control_point_index: usize,
}

impl State {
    pub fn default_from_frequency_transform(t: &params::FrequencyTransform) -> Self {
        Self {
            vowels_map: vec![
                (String::from("U"), t.xy_from_frequencies([320_f32, 800_f32])),
                (
                    String::from("O"),
                    t.xy_from_frequencies([500_f32, 1000_f32]),
                ),
                (
                    String::from("A"),
                    t.xy_from_frequencies([1000_f32, 1400_f32]),
                ),
                (
                    String::from("Ö"),
                    t.xy_from_frequencies([500_f32, 1500_f32]),
                ),
                (
                    String::from("Ü"),
                    t.xy_from_frequencies([320_f32, 1650_f32]),
                ),
                (
                    String::from("Ä"),
                    t.xy_from_frequencies([800_f32, 2000_f32]),
                ),
                (
                    String::from("E"),
                    t.xy_from_frequencies([470_f32, 2300_f32]),
                ),
                (
                    String::from("I"),
                    t.xy_from_frequencies([260_f32, 3200_f32]),
                ),
            ],
            hovered_control_point_index: usize::MAX,
        }
    }
}
