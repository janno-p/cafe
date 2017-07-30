use cqrs::Aggregate;
use std::iter::FromIterator;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    OpenTab(Uuid, u8, String),
    PlaceOrder(Uuid, Vec<OrderedItem>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    TabNotOpen
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    TabOpened { table_number: u8, waiter: String },
    DrinksOrdered { items: Vec<OrderedItem> },
    FoodOrdered { items: Vec<OrderedItem> }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    tab_open: bool
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OrderedItem {
    menu_number: i32,
    description: String,
    is_drink: bool,
    price: f32 // TODO: use decimal
}

pub struct Tab;

impl Aggregate for Tab {
    type Command = Command;
    type CommandError = CommandError;
    type Event = Event;
    type State = State;

    fn initial_state() -> State {
        State {
            tab_open: false
        }
    }

    fn decide(state: &State, command: Command) -> Result<Vec<Event>, CommandError> {
        use self::Command::*;
        use self::CommandError::*;
        use self::Event::*;

        match command {
            OpenTab(_, table_number, waiter) => Ok(vec![TabOpened { table_number, waiter }]),
            PlaceOrder(_, items) => {
                if state.tab_open {
                    let (drinks, foods): (Vec<OrderedItem>, Vec<OrderedItem>) = items.into_iter().partition(|ref n| n.is_drink);
                    let mut events = vec![];
                    if foods.len() > 0 {
                        events.push(FoodOrdered { items: foods });
                    }
                    if drinks.len() > 0 {
                        events.push(DrinksOrdered { items: drinks });
                    }
                    Ok(events)
                } else {
                    Err(TabNotOpen)
                }
            },
            _ => Ok(vec![])
        }
    }

    fn evolve(state: &mut State, event: Event) {
        use self::Event::*;

        match event {
            TabOpened { .. } => state.tab_open = true,
            _ => {}
        }
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

    #[test]
    fn can_not_order_with_unopened_tab() {
        let state = Tab::initial_state();
        let command = Command::PlaceOrder(Uuid::new_v4(), vec![ OrderedItem { menu_number: 0, description: String::new(), is_drink: true, price: 0.0 } ]);
        let events = Tab::decide(&state, command);
        assert_eq!(events, Err(CommandError::TabNotOpen));
    }

    #[test]
    fn can_place_drinks_order() {
        let mut state = Tab::initial_state();
        Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: String::from("Derek") });
        let drink1 = OrderedItem { menu_number: 0, description: String::from(""), is_drink: true, price: 0.0 };
        let drink2 = OrderedItem { menu_number: 0, description: String::from(""), is_drink: true, price: 0.0 };
        let command = Command::PlaceOrder(Uuid::new_v4(), vec![drink1.clone(), drink2.clone()]);
        let events = Tab::decide(&state, command);
        assert_eq!(events, Ok(vec![Event::DrinksOrdered { items: vec![drink1, drink2] }]));
    }

    #[test]
    fn can_place_food_order() {
        let mut state = Tab::initial_state();
        Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: String::from("Derek") });
        let food1 = OrderedItem { menu_number: 0, description: String::from(""), is_drink: false, price: 0.0 };
        let food2 = OrderedItem { menu_number: 0, description: String::from(""), is_drink: false, price: 0.0 };
        let command = Command::PlaceOrder(Uuid::new_v4(), vec![food1.clone(), food2.clone()]);
        let events = Tab::decide(&state, command);
        assert_eq!(events, Ok(vec![Event::FoodOrdered { items: vec![food1, food2] }]));
    }

    #[test]
    fn can_place_food_and_drink_order() {
        let mut state = Tab::initial_state();
        Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: String::from("Derek") });
        let food = OrderedItem { menu_number: 0, description: String::from(""), is_drink: false, price: 0.0 };
        let drink = OrderedItem { menu_number: 0, description: String::from(""), is_drink: true, price: 0.0 };
        let command = Command::PlaceOrder(Uuid::new_v4(), vec![food.clone(), drink.clone()]);
        let events = Tab::decide(&state, command);
        assert_eq!(events, Ok(vec![Event::FoodOrdered { items: vec![food] }, Event::DrinksOrdered { items: vec![drink] }]));
    }
}
