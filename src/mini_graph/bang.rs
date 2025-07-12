#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bang {
    Bang,
    BangF32(f32),
    BangU32(u32),
    BangBool(bool),
    BangUSize(usize),
    SetParamU32(usize, u32),
    SetParamF32(usize, f32),
    SetParamBool(usize, bool),
    Empty,
}