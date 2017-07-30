use cqrs::Aggregate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    OpenTab(Uuid, u8, String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {

}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    TabOpened { table_number: u8, waiter: String }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {

}

pub struct Tab;

impl Aggregate for Tab {
    type Command = Command;
    type CommandError = CommandError;
    type Event = Event;
    type State = Option<State>;

    fn initial_state() -> Option<State> {
        None
    }

    fn decide(state: &Option<State>, command: Command) -> Result<Vec<Event>, CommandError> {
        use self::Command::*;
        use self::Event::*;

        match command {
            OpenTab(_, table_number, waiter) => Ok(vec![TabOpened { table_number, waiter }]),
            _ => Ok(vec![])
        }
    }

    fn evolve(state: &mut Option<State>, event: Event) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_open_a_new_tab() {
        let state = Tab::initial_state();
        let command = Command::OpenTab(Uuid::new_v4(), 42, "Derek".to_string());
        let events = Tab::decide(&state, command);
        assert_eq!(events, Ok(vec![Event::TabOpened { table_number: 42, waiter: "Derek".to_string() } ]));
    }
}
