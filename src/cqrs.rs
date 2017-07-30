pub trait Aggregate {
    type Command;
    type CommandError;
    type State;
    type Event;
    fn initial_state() -> Self::State;
    fn decide(state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::CommandError>;
    fn evolve(state: &mut Self::State, event: Self::Event);
}
