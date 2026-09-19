use enum_table::Enumerable;

#[derive(Debug, PartialEq, Clone, Copy, Enumerable, serde::Serialize, serde::Deserialize)]
pub enum MultibandType {
    Sequential,
    ParallelSum,
    ParallelAverage,
}

impl MultibandType {
    pub const ALL: &'static [MultibandType] = Enumerable::VARIANTS;
    pub const VARIANT_COUNT: usize = Self::COUNT;

    pub const ALL_NAMES: [&'static str; Self::COUNT] =
        ["Sequential", "Parallel Sum", "Parallel Average"];
    pub fn to_string(&self) -> &str {
        Self::ALL_NAMES[*self as usize]
    }
}

impl TryFrom<usize> for MultibandType {
    type Error = &'static str;

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index < Self::COUNT {
            Ok(Self::ALL[index])
        } else {
            Err(stringify!(
                "MultibandType for index {} is not defined",
                index
            ))
        }
    }
}

impl TryFrom<&str> for MultibandType {
    type Error = &'static str;

    fn try_from(type_name: &str) -> Result<Self, Self::Error> {
        let index_option = Self::ALL_NAMES.iter().position(|&name| name == type_name);
        match index_option {
            Some(index) => Ok(Self::ALL[index]),
            None => Err(stringify!("MultibandType {} is not defined", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::eq::*;

    #[test]
    fn multiband_type_from_and_into_string() {
        let round_trip_string = |name: &str| {
            let mb_type = MultibandType::try_from(name).unwrap();
            let name_back = mb_type.to_string();
            assert_eq!(name, name_back);
        };

        let round_trip_multiband_type = |mb_type: MultibandType| {
            let name = mb_type.to_string();
            let mb_type_back = MultibandType::try_from(name).unwrap();
            assert_eq!(mb_type, mb_type_back);
        };

        for type_name in MultibandType::ALL_NAMES {
            round_trip_string(type_name);
        }

        for mb_type in MultibandType::ALL {
            round_trip_multiband_type(*mb_type);
        }
    }
}
