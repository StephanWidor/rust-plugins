use super::*;
use enum_table::Enumerable;

#[derive(nice::Params)]
pub struct Modulation {
    #[id = "Amplitude"]
    pub amplitude: nice::FloatParam,

    #[id = "Length unit"]
    pub length_unit: LengthUnitParam,

    #[id = "Length in seconds"]
    pub length_in_seconds: nice::FloatParam,

    #[nested(group = "Length in Notes")]
    pub length_in_notes: NoteLength,

    pub current_value: nice::AtomicF32,
}

impl Modulation {
    pub fn default_with_name_prefix(prefix: &str) -> Self {
        let common_prefix = format!("{prefix}modulation ");
        Self {
            amplitude: nice::FloatParam::new(
                format!("{common_prefix}amplitude"),
                0_f32,
                nice::FloatRange::Linear {
                    min: -1_f32,
                    max: 1_f32,
                },
            ),
            length_unit: LengthUnitParam::new(
                format!("{common_prefix}length unit"),
                LengthUnit::Note,
            ),
            length_in_seconds: nice::FloatParam::new(
                format!("{common_prefix}length seconds"),
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0.1_f32,
                    max: 5_f32,
                },
            ),
            length_in_notes: NoteLength {
                numerator: nice::IntParam::new(
                    format!("{common_prefix} length note numerator"),
                    1,
                    nice::IntRange::Linear { min: 1, max: 32 },
                ),
                denominator: nice::IntParam::new(
                    format!("{common_prefix} length note denominator"),
                    4,
                    nice::IntRange::Linear { min: 1, max: 64 },
                ),
            },
            current_value: nice::AtomicF32::new(0_f32),
        }
    }
}

#[derive(nice::Params)]
pub struct NoteLength {
    #[id = "Numerator"]
    pub numerator: nice::IntParam,

    #[id = "Denominator"]
    pub denominator: nice::IntParam,
}

#[derive(Debug, PartialEq, Clone, Copy, Enumerable, serde::Serialize, serde::Deserialize)]
pub enum LengthUnit {
    Note,
    Seconds,
}

impl nice::Enum for LengthUnit {
    fn variants() -> &'static [&'static str] {
        &["Note", "Seconds"]
    }

    fn ids() -> Option<&'static [&'static str]> {
        None
    }

    fn to_index(self) -> usize {
        self as usize
    }

    fn from_index(index: usize) -> Self {
        let i = if index < Self::COUNT { index } else { 0 };
        Self::VARIANTS[i]
    }
}

pub type LengthUnitParam = nice::EnumParam<LengthUnit>;
