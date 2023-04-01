use rustfft::num_complex::Complex32;

pub trait Lerpable<Bound> {
    fn lerp(&self, from: Bound, to: Bound) -> Bound;
    fn invlerp(&self, from: Bound, to: Bound) -> Bound;
}
impl Lerpable<f32> for f32 {
    fn lerp(&self, from: f32, to: f32) -> Self {
        from * (1.0 - self) + to * self
    }
    fn invlerp(&self, from: f32, to: f32) -> Self {
        (self - from) / (to - from)
    }
}
impl Lerpable<Complex32> for f32 {
    fn lerp(&self, from: Complex32, to: Complex32) -> Complex32 {
        from * (1.0 - self) + to * self
    }
    fn invlerp(&self, from: Complex32, to: Complex32) -> Complex32 {
        (self - from) / (to - from)
    }
}