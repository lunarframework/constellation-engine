pub struct App {
    runner: Box<dyn Fn(App)>,
}

impl App {
    pub fn empty() -> App {
        Self {
            runner: Box::new(run_once),
        }
    }

    pub fn run(mut self) {
        let runner = std::mem::replace(&mut self.runner, Box::new(run_once));
        (runner)(self);
    }
}

fn run_once(mut _app: App) {
    // app.update();
}
