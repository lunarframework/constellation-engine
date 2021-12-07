pub mod menu;
pub mod panel;

pub mod home;
pub mod initial;

pub use menu::MenuBar;
pub use panel::MainPanel;

pub use home::HomeState;
pub use initial::InitialDataState;

use egui::{CtxRef, Ui};

pub struct StateManager {
    pending_state: Option<Box<dyn State>>,
    state: Box<dyn State>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            pending_state: None,
            state: Box::new(HomeState::new()),
        }
    }

    pub fn enqueue(&mut self, state: impl State) {
        self.pending_state = Some(Box::new(state));
    }

    pub fn view(&mut self, ui: &mut Ui) {
        self.state.view(ui);
    }

    pub fn show(&mut self, ctx: &CtxRef) {
        self.state.show(ctx);
    }

    pub fn update(&mut self) {
        if let Some(state) = self.pending_state.take() {
            self.state = state;
        }

        self.state.update();
    }

    pub fn title(&mut self) -> &str {
        self.state.title()
    }
}

pub trait State: 'static {
    /// Renders the main interface of this state. Everything is put on the main panel.
    fn view(&mut self, ui: &mut Ui);
    /// Render any additional windows and interfaces
    fn show(&mut self, ctx: &CtxRef);
    /// Performs additional calculations and syncronization seperate from the Ui.
    fn update(&mut self);
    /// Title of the window in this state
    fn title(&mut self) -> &str;
}
