use super::*;

#[derive(nice::Params)]
pub struct Modulation {
    #[id = "Length in seconds"]
    pub wave_length: nice::FloatParam,

    #[id = "Amplitude"]
    pub amplitude: nice::FloatParam,

    pub current: nice::AtomicF32,
}
