use cqrs::Aggregate;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    OpenTab(Uuid, u8, String),
    PlaceOrder(Uuid, Vec<OrderedItem>),
    MarkDrinksServed(Uuid, Vec<i32>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    TabNotOpen,
    DrinksNotOutstanding
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    TabOpened { table_number: u8, waiter: String },
    DrinksOrdered { items: Vec<OrderedItem> },
    FoodOrdered { items: Vec<OrderedItem> },
    DrinksServed { menu_numbers: Vec<i32> }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    tab_open: bool,
    outstanding_drinks: Vec<i32>
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
            tab_open: false,
            outstanding_drinks: Vec::new()
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

                    if !foods.is_empty() {
                        events.push(FoodOrdered { items: foods });
                    }

                    if !drinks.is_empty() {
                        events.push(DrinksOrdered { items: drinks });
                    }

                    Ok(events)
                } else {
                    Err(TabNotOpen)
                }
            },
            MarkDrinksServed(_, menu_numbers) => {
                if !state.are_drinks_outstanding(&menu_numbers) {
                    return Err(DrinksNotOutstanding);
                }

                Ok(vec![DrinksServed { menu_numbers: menu_numbers }])
            },
            _ => Ok(vec![])
        }
    }

    fn evolve(state: &mut State, event: Event) {
        use self::Event::*;

        match event {
            TabOpened { .. } => state.tab_open = true,
            DrinksOrdered { items } => {
                let mut menu_numbers: Vec<i32> = items.iter().map(|d| d.menu_number).collect();
                state.outstanding_drinks.append(&mut menu_numbers);
            },
            DrinksServed { menu_numbers } => {
                for menu_number in menu_numbers.iter() {
                    state.outstanding_drinks.remove_item(menu_number);
                }
            },
            _ => {}
        }
    }
}

impl State {
    fn are_drinks_outstanding(&self, menu_numbers: &Vec<i32>) -> bool {
        let mut current_outstanding_drinks = self.outstanding_drinks.clone();

        for menu_number in menu_numbers.iter() {
            if current_outstanding_drinks.contains(menu_number) {
                current_outstanding_drinks.remove_item(menu_number);
            } else {
                return false;
            }
        }

        true
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

    #[test]
    fn ordered_drinks_can_be_served() {
        let mut state = Tab::initial_state();
        Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: "Derek".to_string() });
        let drink1 = OrderedItem { menu_number: 1, description: "".to_string(), is_drink: true, price: 0.0 };
        let drink2 = OrderedItem { menu_number: 2, description: "".to_string(), is_drink: true, price: 0.0 };
        Tab::evolve(&mut state, Event::DrinksOrdered { items: vec![drink1.clone(), drink2.clone()] });
        let command = Command::MarkDrinksServed(Uuid::new_v4(), vec![drink1.menu_number, drink2.menu_number]);
        let events = Tab::decide(&state, command);
        assert_eq!(events, Ok(vec![Event::DrinksServed { menu_numbers: vec![drink1.menu_number, drink2.menu_number] }]));
    }

    #[test]
    fn can_not_serve_an_unordered_drink() {
         let mut state = Tab::initial_state();
         Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: "Derek".to_string() });
         let drink1 = OrderedItem { menu_number: 1, description: "".to_string(), is_drink: true, price: 0.0 };
         let drink2 = OrderedItem { menu_number: 2, description: "".to_string(), is_drink: true, price: 0.0 };
         Tab::evolve(&mut state, Event::DrinksOrdered { items: vec![drink1.clone()] });
         let command = Command::MarkDrinksServed(Uuid::new_v4(), vec![drink2.menu_number]);
         let events = Tab::decide(&state, command);
         assert_eq!(events, Err(CommandError::DrinksNotOutstanding));
    }

    #[test]
    fn can_not_serve_an_ordered_drink_twice() {
         let mut state = Tab::initial_state();
         Tab::evolve(&mut state, Event::TabOpened { table_number: 42, waiter: "Derek".to_string() });
         let drink = OrderedItem { menu_number: 1, description: "".to_string(), is_drink: true, price: 0.0 };
         Tab::evolve(&mut state, Event::DrinksOrdered { items: vec![drink.clone()] });
         Tab::evolve(&mut state, Event::DrinksServed { menu_numbers: vec![drink.menu_number] });
         let command = Command::MarkDrinksServed(Uuid::new_v4(), vec![drink.menu_number]);
         let events = Tab::decide(&state, command);
         assert_eq!(events, Err(CommandError::DrinksNotOutstanding));
    }
}
