use crate::{query::Query, world::World};

pub struct System {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn FnMut(&mut World)>,
}

impl System {
    pub fn new<T: 'static>(name: &str, func: fn(Query<T>)) -> System {
        System {
            name: name.to_string(),
            func_ptr: func as *const (),
            exec: Box::new(move |world| {
                let query = Query::new(world);
                func(query);
            }),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        (self.exec)(world);
    }
}
