use super::{System, SystemNodeChildren};
use std::any::Any;
pub trait Solver: Any {
    fn solve<S: System>(&mut self, system: &mut S, children: &mut SystemNodeChildren);
}

pub struct EmptySolver;

impl Solver for EmptySolver {
    fn solve<S: System>(&mut self, _system: &mut S, _children: &mut SystemNodeChildren) {}
}
