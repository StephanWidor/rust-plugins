use enum_table::Enumerable;

#[derive(Debug, PartialEq, Clone, Copy, Enumerable, serde::Serialize, serde::Deserialize)]
pub enum EqType {
    Volume,
    LowPass,
    HighPass,
    BandPass,
    AllPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
    Bypassed,
}

impl EqType {
    pub const ALL: &'static [EqType] = Enumerable::VARIANTS;
    pub const VARIANT_COUNT: usize = Self::COUNT;

    pub const ALL_NAMES: [&'static str; Self::COUNT] = [
        "Volume",
        "Low Pass",
        "High Pass",
        "Band Pass",
        "AllPass",
        "Notch",
        "Peak",
        "Low Shelf",
        "High Shelf",
        "Bypassed",
    ];
    pub fn to_string(&self) -> &str {
        Self::ALL_NAMES[*self as usize]
    }

    pub const fn is_active(&self) -> bool {
        match self {
            EqType::Bypassed => false,
            _ => true,
        }
    }

    pub const fn has_frequency(&self) -> bool {
        match self {
            EqType::Volume => false,
            EqType::Bypassed => false,
            _ => true,
        }
    }

    pub const fn has_gain_db(&self) -> bool {
        match self {
            EqType::Volume => true,
            EqType::Peak => true,
            EqType::LowShelf => true,
            EqType::HighShelf => true,
            _ => false,
        }
    }

    pub const fn has_q(&self) -> bool {
        match self {
            EqType::Volume => false,
            EqType::Bypassed => false,
            _ => true,
        }
    }
}

impl TryFrom<usize> for EqType {
    type Error = &'static str;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index < Self::COUNT {
            Ok(Self::ALL[index])
        } else {
            Err(stringify!("EqType for index {} is not defined", index))
        }
    }
}

impl TryFrom<&str> for EqType {
    type Error = &'static str;

    fn try_from(type_name: &str) -> Result<Self, Self::Error> {
        let index_option = Self::ALL_NAMES.iter().position(|&name| name == type_name);
        match index_option {
            Some(index) => Ok(Self::ALL[index]),
            None => Err(stringify!("EqType {} is not defined", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::eq::*;

    #[test]
    fn eq_type_from_and_into_string() {
        let round_trip_string = |name: &str| {
            let eq_type = EqType::try_from(name).unwrap();
            let name_back = eq_type.to_string();
            assert_eq!(name, name_back);
        };

        let round_trip_eq_type = |eq_type: EqType| {
            let name = eq_type.to_string();
            let eq_type_back = EqType::try_from(name).unwrap();
            assert_eq!(eq_type, eq_type_back);
        };

        for eq_type_name in EqType::ALL_NAMES {
            round_trip_string(eq_type_name);
        }

        for eq_type in EqType::ALL {
            round_trip_eq_type(*eq_type);
        }
    }

    #[test]
    fn eq_from_and_into() {
        let eq_f32 = Eq {
            gain: Gain::Db(-3.0_f32),
            frequency: Frequency::Hz(440.0_f32),
            q: 0.707_f32,
            eq_type: EqType::Peak,
            makeup_gain: Gain::Amplitude(2_f32),
        };
        let eq_f64: Eq<f64> = Eq::<f64>::from(eq_f32.clone());

        let eq_f32_back: Eq<f32> = eq_f64.into();
        assert_eq!(eq_f32, eq_f32_back);
    }
}
