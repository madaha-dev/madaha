use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum WaveType {
    Saw = 0,
    Triangle = 1,
    Square = 2,
    Random = 3,
    Sine = 4,
    SawEG = 5,
    TriangleEG = 6,
    SquareEG = 7,
    RandomEG = 8,
    SawAlt = 9,
    TriangleAlt = 10,
    SquareAlt = 11,
    RandomAlt = 12,
}

#[derive(Debug, PartialEq)]
pub enum WaveVariation {
    Normal,
    EG,
    Alt,
}
