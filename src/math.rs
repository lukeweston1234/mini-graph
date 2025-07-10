#[inline(always)]
pub fn lerp(a: f32, b: f32, x: f32) -> f32{
    a + (b - a) * x
}

#[inline(always)]
pub fn delerp(a: f32, b: f32, y: f32) -> f32 {
    (y - a) / (b - a)
}