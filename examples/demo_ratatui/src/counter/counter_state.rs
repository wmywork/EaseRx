use easerx::State;

#[derive(Clone, Debug, PartialEq)]
pub struct CounterState {
    pub count: i32,
    pub started: bool,
}

impl State for CounterState {}

impl Default for CounterState {
    fn default() -> Self {
        Self {
            count: 0,
            started: false,
        }
    }
}
impl CounterState {
    pub fn on_counter_increment(self) -> Self {
        Self {
            count: self.count + 1,
            ..self
        }
    }
}
