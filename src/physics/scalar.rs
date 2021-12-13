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
    pub phi: Array2<f64>,
    pub psi: Array2<f64>,
    pub pi: Array2<f64>,
    pub lapse: Array2<f64>,
    pub metric: Array2<f64>,
    pub rmax: f64,
    pub tend: f64,
}

pub fn simulate(data: InitialData, mut update_callback: impl FnMut(f64)) -> Simulation {
    assert!(data.wave.len() > 4);
    // Retrieve field and domain data
    let rmax = data.rmax;
    let nr = data.wave.len();

    let tend = data.tend;
    let iterations = data.iterations;
    let dt = tend / iterations as f64;

    // Mesh
    let mesh = Mesh::new(rmax, nr);

    // ********************
    // Initial Data *******
    // ********************

    let mass = data.mass;

    let mut phi = data.wave;
    let mut psi = Function1::zeros(nr);
    mesh.gradient(&phi, &mut psi);
    psi[0] = 0.0; //dr (phi) = 0 at r = 0
    let mut pi = Function1::zeros(nr);

    let mut dphi = Function1::zeros(nr);
    let mut dpsi = Function1::zeros(nr);
    let mut dpi = Function1::zeros(nr);

    let mut dphi_prev = Function1::zeros(nr);
    let mut dpsi_prev = Function1::zeros(nr);
    let mut dpi_prev = Function1::zeros(nr);

    // *******************
    // Metric ************
    // *******************

    let mut lapse = Function1::zeros(nr);
    let mut metric = Function1::zeros(nr);

    // ******************
    // Storage **********
    // ******************

    let mut phi_storage = Array2::zeros([data.iterations, nr]);
    let mut psi_storage = Array2::zeros([data.iterations, nr]);
    let mut pi_storage = Array2::zeros([data.iterations, nr]);
    let mut metric_storage = Array2::zeros([data.iterations, nr]);
    let mut lapse_storage = Array2::zeros([data.iterations, nr]);

    // Loop

    for i in 0..iterations {
        // *******************
        // Setup *************
        // *******************

        let potential = phi.map(|v| 0.5 * mass * mass * v * v);

        // *******************
        // Update Metric *****
        // *******************

        // Update lapse by integrating from center outwards

        // dmetric/dr = metric * (1 - metric^2) / (2 * r) + metric * r / 4 * (psi^2 + pi^2 + 2 * metric^2 * potential)
        let metric_derivative = |r, metric, psi, pi, potential| {
            let sq_metric = metric * metric;
            let sq_psi = psi * psi;
            let sq_pi = pi * pi;
            let term0 = (1.0 - sq_metric) / (2.0 * r);
            let term1 = r / 4.0 * (sq_psi + sq_pi + 2.0 * sq_metric * potential);
            metric * (term0 + term1)
        };

        // Assume Spatial flatness at origin
        metric[0] = 1.0;
        // sq_metric = 1 so dmetric = 0.0 at r = 0
        let mut dmetric_prev = 0.0;
        // Thus, by Euler's method, the next metric value is also one
        metric[1] = 1.0;

        for i in 1..(mesh.nr - 1) {
            let dmetric = metric_derivative(mesh.r[i], metric[i], psi[i], pi[i], potential[i]);
            // println!(
            //     "Dmetric {}, r {}, psi {}, pi {}, potential {}",
            //     dmetric, mesh.r[i], psi[i], pi[i], potential[i]
            // );

            metric[i + 1] = metric[i] + mesh.dr * (1.5 * dmetric - 0.5 * dmetric_prev);

            dmetric_prev = dmetric;
        }

        // *******************
        // Update Lapse ******
        // *******************

        // Update lapse by integrating from outside to center

        let lapse_derivative = |r, metric, lapse, psi, pi, potential| {
            // **************
            let sq_metric = metric * metric;
            let sq_psi = psi * psi;
            let sq_pi = pi * pi;
            // **************
            let term0 = (1.0 - sq_metric) / (2.0 * r)
                + r / 4.0 * (sq_psi + sq_pi + 2.0 * sq_metric * potential);
            let term1 = (sq_metric - 1.0) / r;
            let term2 = r * sq_metric * potential;

            lapse * (term0 + term1 - term2)
        };

        lapse[mesh.nr - 1] = 1.0 / metric[mesh.nr - 1];
        let mut dlapse_prev = 0.0;
        for i in (1..mesh.nr).rev() {
            let dlapse =
                lapse_derivative(mesh.r[i], metric[i], lapse[i], psi[i], pi[i], potential[i]);

            if i == mesh.nr - 1 {
                lapse[i - 1] = lapse[i] - mesh.dr * dlapse;
            } else {
                lapse[i - 1] = lapse[i] - mesh.dr * (1.5 * dlapse - 0.5 * dlapse_prev);
            };

            dlapse_prev = dlapse;
        }

        // println!("Metric {}", metric);
        // println!("Lapse {}", lapse);

        // ******************
        // Update dphi ******
        // ******************

        // dphi/dt = lapse / metric * pi
        Zip::from(&mut dphi)
            .and(&metric)
            .and(&lapse)
            .and(&pi)
            .for_each(|dphi, &metric, &lapse, &pi| {
                *dphi = lapse / metric * pi;
            });

        // *****************
        // Update dpsi *****
        // *****************

        // dpsi/dt = d/dr (lapse / metric * pi)
        mesh.gradient(&dphi, &mut dpsi);

        // *****************
        // Update dpi ******
        // *****************

        let mut term = Function1::zeros(nr);
        let mut gterm = Function1::zeros(nr);

        Zip::from(&mut term)
            .and(&lapse)
            .and(&metric)
            .and(&psi)
            .for_each(|t, &lapse, &metric, &psi| {
                *t = lapse * psi / metric;
            });

        mesh.gradient(&term, &mut gterm);

        for i in 1..nr {
            dpi[i] = gterm[i] + 2.0 / mesh.r[i] * term[i] - lapse[i] * metric[i] * phi[i];
            // dpi_diff[i] = 1.0 / (mesh.r[i] * mesh.r[i]) * gterm[i] - lapse[i] * metric[i] * phi[i];
        }

        let mut dpi_diff = Function1::zeros(nr);

        let mut term_diff = Function1::zeros(nr);
        let mut gterm3_diff = Function1::zeros(nr);
        // let mut gterm_diff = Function1::zeros(nr);

        Zip::from(&mut term_diff)
            .and(&lapse)
            .and(&metric)
            .and(&psi)
            .and(&mesh.r)
            .for_each(|t, &lapse, &metric, &psi, &r| {
                *t = r * r * lapse * psi / metric;
            });

        mesh.gradient3(&term_diff, &mut gterm3_diff);
        // mesh.gradient(&term_diff, &mut gterm_diff);

        for i in 0..nr {
            dpi_diff[i] = 3.0 * gterm3_diff[i] - lapse[i] * metric[i] * phi[i];
            // dpi_diff[i] =
            //     1.0 / (mesh.r[i] * mesh.r[i]) * gterm_diff[i] - lapse[i] * metric[i] * phi[i];
        }

        // let d2_dr2_phi = (2.0 * phi[1] - 2.0 * phi[0]) / (mesh.dr * mesh.dr);

        // ****************
        // Rhs Boundry ****
        // ****************

        dpsi[0] = 0.0;

        let d2phi_dr2 = psi[1] / mesh.dr;

        dpi[0] = 3.0 * lapse[0] * d2phi_dr2 - lapse[0] * phi[0];

        // ****************
        // Storage ********
        // ****************

        for rindex in 0..nr {
            phi_storage[(i, rindex)] = phi[rindex];
            psi_storage[(i, rindex)] = psi[rindex];
            pi_storage[(i, rindex)] = pi[rindex];
            metric_storage[(i, rindex)] = metric[rindex];
            lapse_storage[(i, rindex)] = lapse[rindex];
        }

        // ***************
        // Evolve ********
        // ***************

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

        // ****************
        // Rhs Boundry ****
        // ****************

        // psi[mesh.nr - 1] = -pi[mesh.nr - 1] - phi[mesh.nr - 1] / mesh.r[mesh.nr - 1];

        update_callback(i as f64 * dt);
    }

    // for rindex in 0..nr {
    //     phi_storage[(iterations, rindex)] = phi[rindex];
    //     psi_storage[(iterations, rindex)] = psi[rindex];
    //     pi_storage[(iterations, rindex)] = pi[rindex];
    //     metric_storage[(iterations, rindex)] = metric[rindex];
    //     lapse_storage[(iterations, rindex)] = lapse[rindex];
    // }

    Simulation {
        phi: phi_storage,
        psi: psi_storage,
        pi: pi_storage,
        metric: metric_storage,
        lapse: lapse_storage,
        rmax,
        tend,
    }
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

    fn gradient(&self, f: &Function1, dst: &mut Function1) {
        assert_eq!(f.len(), self.nr);
        assert_eq!(f.len(), dst.len());

        let len = f.len();

        Zip::from(dst.slice_mut(s![1..(len - 1)]))
            .and(f.slice(s![2..]))
            .and(f.slice(s![..(len - 2)]))
            .for_each(|dst, &f1, &f0| {
                *dst = (f1 - f0) / (2.0 * self.dr);
            });

        dst[0] = (-f[2] + 4.0 * f[1] - 3.0 * f[0]) / (2.0 * self.dr);
        dst[len - 1] = (f[len - 3] - 4.0 * f[len - 2] + 3.0 * f[len - 1]) / (2.0 * self.dr);
    }

    fn gradient3(&self, f: &Function1, dst: &mut Function1) {
        assert_eq!(f.len(), self.nr);
        assert_eq!(f.len(), dst.len());

        let len = f.len();

        let du = self.dr * self.dr * self.dr;

        for i in 1..(len - 1) {
            let m = i as f64;

            let lhs = crate::math::interp(
                f[i - 1],
                f[i],
                crate::math::interp_fraction(m - 1.0, m, (m * m * m - 1.0).powf(1.0 / 3.0)),
            );
            let rhs = crate::math::interp(
                f[i],
                f[i + 1],
                crate::math::interp_fraction(m, m + 1.0, (m * m * m + 1.0).powf(1.0 / 3.0)),
            );

            dst[i] = (rhs - lhs) / (2.0 * du);
        }

        {
            let f0 = f[0];
            let f1 = f[1];
            let f2 = crate::math::interp(
                f[1],
                f[2],
                crate::math::interp_fraction(1.0, 2.0, (2.0 * du).powf(1.0 / 3.0)),
            );

            dst[0] = (-f2 + 4.0 * f1 - 3.0 * f0) / (2.0 * du);
        }
    }

    // fn gradient(&self, f: &Function1) -> Function1 {
    //     assert_eq!(f.len(), self.nr);

    //     let mut gradient = Function1::zeros(self.nr);

    //     let len = f.len();

    //     Zip::from(gradient.slice_mut(s![1..(len - 1)]))
    //         .and(f.slice(s![2..]))
    //         .and(f.slice(s![..(len - 2)]))
    //         .for_each(|dst, &f1, &f0| {
    //             *dst = (f1 - f0) / (2.0 * self.dr);
    //         });
    //     gradient[0] = (-f[2] + 4.0 * f[1] - 3.0 * f[0]) / (2.0 * self.dr);
    //     gradient[len - 1] = (f[len - 3] - 4.0 * f[len - 2] + 3.0 * f[len - 1]) / (2.0 * self.dr);

    //     // gradient[0] = (f[1] - f[0]) / self.dr;
    //     // gradient[len - 1] = (f[len - 2] - f[len - 1]) / self.dr;

    //     gradient
    // }
}
