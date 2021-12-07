use ndarray::{s, Array1, Array2, Zip};

pub type Function1 = ndarray::Array1<f64>;

pub struct InitialData {
    pub mass: f64,
    pub rmax: f64,
    pub wave: Function1,

    pub tend: f64,
    pub iterations: usize,
    // pub storage_points: usize,
    // pub timestamps: usize,
}

pub struct Simulation {
    pub wave: Array2<f64>,
    pub rmax: f64,
    pub tend: f64,
}

pub fn simulate(data: InitialData, mut update_callback: impl FnMut(f64)) -> Simulation {
    assert!(data.wave.len() > 4);
    // Retrieve field and domain data
    let rmax = data.rmax;
    let nr = data.wave.len();

    let dr = rmax / (nr - 1) as f64;

    let tend = data.tend;
    let iterations = data.iterations;
    let dt = tend / (iterations) as f64;

    // Mesh
    let mesh = Mesh::new(rmax, nr);

    // ********************
    // Initial Data *******
    // ********************

    let mass = data.mass;

    let mut phi = data.wave;
    let mut psi = mesh.gradient(&phi);
    let mut pi = Function1::zeros(nr);

    let mut dphi = Function1::zeros(nr);
    // let mut dpsi;
    let mut dpi = Function1::zeros(nr);

    let mut dphi_prev = Function1::zeros(nr);
    let mut dpsi_prev = Function1::zeros(nr);
    let mut dpi_prev = Function1::zeros(nr);

    // *******************
    // Metric ************
    // *******************

    let mut lapse = Function1::zeros(nr);
    let mut metric = Function1::zeros(nr);
    let mut dlapse = Function1::zeros(nr);
    let mut dmetric = Function1::zeros(nr);

    // ******************
    // Storage **********
    // ******************

    let mut wave = Array2::zeros([data.iterations + 1, nr]);

    // Loop

    for i in 0..iterations {
        // ****************
        // Storage ********
        // ****************

        for rindex in 0..nr {
            wave[(i, rindex)] = phi[rindex];
        }

        // *******************
        // Setup *************
        // *******************

        let potential = phi.map(|v| 0.5 * mass * mass * v * v);

        // *******************
        // Update Metric *****
        // *******************

        // Update lapse by integrating from center outwards

        let metric_derivative = |metric, r, psi, pi, potential| {
            let sq_metric = metric * metric;
            let sq_psi = psi * psi;
            let sq_pi = pi * pi;
            metric
                * ((1.0 - sq_metric) / (2.0 * r)
                    + r / 4.0 * (sq_psi + sq_pi + 2.0 * sq_metric * potential))
        };

        metric[0] = 1.0;
        dmetric[0] = 0.0;

        for i in 1..(mesh.nr - 1) {
            dmetric[i] = metric_derivative(metric[i], mesh.r[i], psi[i], pi[i], potential[i]);
            metric[i + 1] = metric[i] + mesh.dr * (1.5 * dmetric[i] - 0.5 * dmetric[i - 1]);
        }

        dmetric[mesh.nr - 1] = metric_derivative(
            metric[mesh.nr - 1],
            mesh.r[mesh.nr - 1],
            psi[mesh.nr - 1],
            pi[mesh.nr - 1],
            potential[mesh.nr - 1],
        );

        // *******************
        // Update Lapse ******
        // *******************

        // Update lapse by integrating from outside to center

        let lapse_derivative = |r, lapse, metric, dmetric, potential| {
            lapse
                * ((dmetric / metric) + (metric * metric - 1.0) / r
                    - r * metric * metric * potential)
        };

        lapse[mesh.nr - 1] = 1.0 / metric[mesh.nr - 1];
        dlapse[mesh.nr - 1] = lapse_derivative(
            mesh.r[mesh.nr - 1],
            lapse[mesh.nr - 1],
            metric[mesh.nr - 1],
            dmetric[mesh.nr - 1],
            potential[mesh.nr - 1],
        );

        for i in (1..(mesh.nr - 1)).rev() {
            dlapse[i] = lapse_derivative(mesh.r[i], lapse[i], metric[i], dmetric[i], potential[i]);

            lapse[i - 1] = lapse[i] + mesh.dr * (1.5 * dlapse[i] - 0.5 * dlapse[i + 1]);
        }

        dlapse[0] = lapse_derivative(mesh.r[0], lapse[0], metric[0], dmetric[0], potential[0]);

        // ******************
        // Update dphi ******
        // ******************

        Zip::from(&mut dphi)
            .and(&metric)
            .and(&lapse)
            .and(&phi)
            .for_each(|dphi, &metric, &lapse, &phi| {
                *dphi = lapse / metric * phi;
            });

        // *****************
        // Update dpsi *****
        // *****************

        let dpsi = mesh.gradient(&dphi);

        // *****************
        // Update dpi ******
        // *****************

        let mut term = Function1::zeros(nr);

        Zip::from(&mut term)
            .and(&mesh.r)
            .and(&dphi)
            .for_each(|t, &r, &d| {
                *t = r * r * d;
            });

        let gterm = mesh.gradient(&term);

        for i in 1..nr {
            dpi[i] = 1.0 / (mesh.r[i] * mesh.r[i]) * gterm[i] - lapse[i] * metric[i] * psi[i];
        }

        const TWO_TO_ONE_THIRD: f64 = 1.25992104989;
        const TWO_TO_TWO_THIRDS: f64 = 1.58740105197;

        let du = dr * dr * dr;

        let f0 = 0.0;
        let f1 = dr * dr * dphi[1];
        let f2 = TWO_TO_TWO_THIRDS * dr * dr * interp(dphi[1], dphi[2], TWO_TO_ONE_THIRD - 1.0);

        dpi[0] = (f2 - 4.0 * f1 + 3.0 * f0) / (2.0 * du);

        if i == 0 {
            // First iteration update with euler's method
            phi.scaled_add(dt, &dphi);
            psi.scaled_add(dt, &dpsi);
            pi.scaled_add(dt, &dpi);
        } else {
            // Otherwise use Adams-Bashforth

            phi.scaled_add(1.5 * dt, &dphi);
            phi.scaled_add(-0.5 * dt, &dphi_prev);

            psi.scaled_add(1.5 * dt, &dpsi);
            psi.scaled_add(-0.5 * dt, &dpsi_prev);

            pi.scaled_add(1.5 * dt, &dpi);
            pi.scaled_add(-0.5 * dt, &dpi_prev);
        }

        dphi_prev.assign(&dphi);
        dpsi_prev.assign(&dpsi);
        dpi_prev.assign(&dpi);

        update_callback(i as f64 * dt);
    }

    for rindex in 0..nr {
        wave[(iterations - 1, rindex)] = phi[rindex];
    }

    Simulation { wave, rmax, tend }
}

fn interp(a: f64, b: f64, x: f64) -> f64 {
    a + (b - a) * x
}

struct Mesh {
    r: Function1,
    dr: f64,
    _rmax: f64,
    nr: usize,
}

impl Mesh {
    fn new(rmax: f64, nr: usize) -> Self {
        let r = Array1::linspace(0.0, rmax, nr);
        let dr = r[1] - r[0];

        Self {
            r,
            dr,
            _rmax: rmax,
            nr,
        }
    }

    fn gradient(&self, f: &Function1) -> Function1 {
        assert_eq!(f.len(), self.nr);

        let mut gradient = Function1::zeros(self.nr);

        let len = f.len();

        Zip::from(gradient.slice_mut(s![1..(len - 1)]))
            .and(f.slice(s![2..]))
            .and(f.slice(s![..(len - 2)]))
            .for_each(|dst, &f1, &f0| {
                *dst = (f1 - f0) / self.dr;
            });
        gradient[0] = (f[2] - 4.0 * f[1] + 3.0 * f[0]) / (2.0 * self.dr);
        gradient[len - 1] = (f[len - 3] - 4.0 * f[len - 2] + 3.0 * f[len - 1]) / (2.0 * self.dr);

        gradient
    }
}
