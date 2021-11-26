use ndarray::{s, Array, Array1, ArrayBase, ArrayView, ArrayView1, ArrayViewMut1, Zip};

struct Mesh {
    r: Array1<f64>,
    dr: f64,
    rmax: f64,
    nr: usize,
}

impl Mesh {
    fn new(rmax: f64, nr: usize) -> Self {
        let r = Array1::linspace(0.0, rmax, nr);
        let dr = r[1] - r[0];

        Self { r, dr, rmax, nr }
    }

    fn gradient(&self, f: &Array1<f64>) -> Array1<f64> {
        assert_eq!(f.len(), self.nr);

        let mut gradient = Array1::<f64>::zeros(self.nr);

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

struct ScalarField {
    phi: Array1<f64>,
    psi: Array1<f64>,
    pi: Array1<f64>,
    potential: Array1<f64>,

    dphi: Array1<f64>,
    dpi: Array1<f64>,
    dpsi: Array1<f64>,

    dphi_prev: Array1<f64>,
    dpi_prev: Array1<f64>,
    dpsi_prev: Array1<f64>,

    dpi_term: Array1<f64>,
}

impl ScalarField {
    fn time_symmetric(phi: Array1<f64>, mesh: &Mesh) -> Self {
        Self {
            psi: mesh.gradient(&phi),
            pi: Array1::<f64>::zeros(phi.len()),
            potential: Array1::<f64>::zeros(phi.len()),
            phi,

            dphi: Array1::<f64>::zeros(mesh.nr),
        }
    }

    fn update(&mut self, mesh: &Mesh, metric: &Metric, dt: f64) {
        Zip::from(&mut self.dphi)
            .and(&metric.metric)
            .and(&metric.lapse)
            .and(&self.phi)
            .for_each(|dphi, &metric, &lapse, &phi| {
                *dphi = lapse / metric * phi;
            });

        Zip::from(&mut self.dpi_term)
            .and(&mesh.r)
            .and(&self.dphi)
            .for_each(|term, &r, &dphi| {
                *term = r * r * dphi;
            });

        // Zip::from(&mut self.dpi).and(mesh.r)
    }
}

struct Metric {
    metric: Array1<f64>,
    dmetric: Array1<f64>,
    lapse: Array1<f64>,
    dlapse: Array1<f64>,
}

impl Metric {
    fn new(mesh: &Mesh, field: &ScalarField) -> Self {
        let mut s = Self {
            metric: Array1::<f64>::zeros(mesh.nr),
            dmetric: Array1::<f64>::zeros(mesh.nr),
            lapse: Array1::<f64>::zeros(mesh.nr),
            dlapse: Array1::<f64>::zeros(mesh.nr),
        };

        s.update(mesh, field);

        s
    }

    fn update(&mut self, mesh: &Mesh, field: &ScalarField) {
        // Update a by integrating from center outwards

        let metric_derivative = |metric, r, psi, pi, potential| {
            let sq_metric = metric * metric;
            let sq_psi = psi * psi;
            let sq_pi = pi * pi;
            metric
                * ((1.0 - sq_metric) / (2.0 * r)
                    + r / 4.0 * (sq_psi + sq_pi + 2.0 * sq_metric * potential))
        };

        self.metric[0] = 1.0;
        self.dmetric[0] = 0.0;

        for i in 1..(mesh.nr - 1) {
            let metric = self.metric[i];
            self.dmetric[i] = metric_derivative(
                metric,
                mesh.r[i],
                field.psi[i],
                field.pi[i],
                field.potential[i],
            );
            self.metric[i + 1] =
                metric + mesh.dr * (1.5 * self.dmetric[i] - 0.5 * self.dmetric[i - 1]);
        }

        self.dmetric[mesh.nr - 1] = metric_derivative(
            self.metric[mesh.nr - 1],
            mesh.r[mesh.nr - 1],
            field.psi[mesh.nr - 1],
            field.pi[mesh.nr - 1],
            field.potential[mesh.nr - 1],
        );

        // Update lapse by integrating from outside to center

        let lapse_derivative = |lapse, metric, dmetric, r, potential| {
            lapse
                * ((dmetric / metric) + (metric * metric - 1.0) / r
                    - r * metric * metric * potential)
        };

        self.lapse[mesh.nr - 1] = 1.0 / self.metric[mesh.nr - 1];
        self.dlapse[mesh.nr - 1] = lapse_derivative(
            self.lapse[mesh.nr - 1],
            self.metric[mesh.nr - 1],
            self.dmetric[mesh.nr - 1],
            mesh.r[mesh.nr - 1],
            field.potential[mesh.nr - 1],
        );

        for i in (1..mesh.nr).rev() {
            let lapse = self.lapse[i];
            self.dlapse[i] = lapse_derivative(
                lapse,
                self.metric[i],
                self.dmetric[i],
                mesh.r[i],
                field.potential[i],
            );

            self.lapse[i - 1] = lapse + mesh.dr * (1.5 * self.dlapse[i] - 0.5 * self.dlapse[i + 1]);
        }

        self.dlapse[0] = lapse_derivative(
            self.lapse[0],
            self.metric[0],
            self.dmetric[0],
            mesh.r[0],
            field.potential[0],
        )
    }
}

fn run() {
    // *********************
    // Config
    // *********************
    let rmax = 1.0;
    let nr = 1000;

    // *********************
    // Coordinates
    // *********************
    let r = Array1::<f64>::linspace(0.0, rmax, nr);
    let dr = r[1] - r[0];

    // *********************
    // Initial Data
    // *********************
}
