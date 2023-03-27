pub trait Lerpable<Bound> {
    fn lerp(&self, from: Bound, to: Bound) -> Self;
    fn invlerp(&self, from: Bound, to: Bound) -> Self;
}
impl Lerpable<f32> for f32 {
    fn lerp(&self, from: f32, to: f32) -> Self {
        from * (1.0 - self) + to * self
    }
    fn invlerp(&self, from: f32, to: f32) -> Self {
        (self - from) / (to - from)
    }
}