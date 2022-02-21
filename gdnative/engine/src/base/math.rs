pub trait AbstractVector: Clone {
    fn zero() -> Self;
    fn one() -> Self;

    fn add<O: AbstractVector>(&mut self, other: O);
    fn scale(&mut self, scalar: f64);

    fn add_scaled<O: AbstractVector>(&mut self, scale: f64, mut other: O) {
        other.scale(scale);
        self.add(other);
    }

    fn lerp(&mut self, mut b: Self, x: f64) {
        self.scale(x);
        b.scale(1.0 - x);
        self.add(b);
    }
}
