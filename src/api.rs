use rocket;

use domain::Event;

pub struct EventStore<T> {
    events: Vec<T>
}

pub fn launch(event_store: EventStore<Event>) {
    let routes = routes![

    ];
    rocket::ignite()
        .mount("/api/", routes)
        .manage(event_store)
        .launch();
}
