use egui::{CtxRef, Ui};

pub struct Dialog<'open> {
    name: String,
    open: &'open mut bool,
}

impl<'open> Dialog<'open> {
    pub fn new(name: impl ToString, open: &'open mut bool) -> Self {
        Self {
            name: name.to_string(),
            open: open,
        }
    }

    /// Shows the dialog. If the `add_contents` function returns false, this sets
    /// `open` to false.
    pub fn show(self, ctx: &CtxRef, add_contents: impl FnOnce(&mut Ui) -> bool) {
        let response = egui::Window::new(self.name)
            .open(self.open)
            .show(ctx, add_contents);

        if let Some(response) = response {
            // Window is open
            if let Some(res) = response.inner {
                // Window is not collapsed
                *self.open &= res;
            }
        }
    }
}
