use ndarray::{s, Array, Array1, ArrayBase, ArrayView, ArrayView1, ArrayViewMut1, Zip};

pub struct InitialData {
    name: String,
    phi: Array1<f64>,
    psi: Array1<f64>,
    pi: Array1<f64>,
}
