use rustfft::num_complex::Complex32;

pub trait Lerpable<Bound> {
    fn lerp(&self, from: Bound, to: Bound) -> Bound;
    fn invlerp(&self, from: Bound, to: Bound) -> Bound;
}
impl Lerpable<f32> for f32 {
    fn lerp(&self, from: f32, to: f32) -> f32 {
        from * (1.0 - self) + to * self
    }
    fn invlerp(&self, from: f32, to: f32) -> f32 {
        (self - from) / (to - from)
    }
}
impl Lerpable<(f32, f32)> for (f32, f32) {
    fn lerp(&self, from: (f32, f32), to: (f32, f32)) -> (f32, f32) {
        (
            from.0 * (1.0 - self.0) + to.0 * self.0,
            from.1 * (1.0 - self.1) + to.1 * self.1,
        )
    }
    fn invlerp(&self, from: (f32,f32), to: (f32,f32)) -> (f32, f32) {
        (
            (self.0 - from.0) / (to.0 - from.0),
            (self.1 - from.1) / (to.1 - from.1),
        )
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