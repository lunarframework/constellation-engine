pub fn interp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn interp_fraction(x0: f64, x1: f64, x: f64) -> f64 {
    (x - x0) / (x1 - x0)
}
