use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;

#[derive(Component)]
struct Toppings {
    value: u32,
}

impl Toppings {
    const BACON: u32 = 0x1;
    const LETTUCE: u32 = 0x2;
    const TOMATO: u32 = 0x4;

    fn new() -> Self {
        Toppings { value: 0 }
    }

    fn add(&mut self, topping: u32) {
        self.value |= topping;
    }

    fn remove(&mut self, topping: u32) {
        self.value &= !topping;
    }

    fn has(&self, topping: u32) -> bool {
        self.value & topping != 0
    }
}

#[derive(Component)]
struct Sandwich {
    toppings: Toppings,
}

impl Sandwich {
    fn new() -> Self {
        Sandwich {
            toppings: Toppings::new(),
        }
    }

    fn add_topping(&mut self, topping: u32) {
        self.toppings.add(topping);
    }

    fn remove_topping(&mut self, topping: u32) {
        self.toppings.remove(topping);
    }

    fn has_topping(&self, topping: u32) -> bool {
        self.toppings.has(topping)
    }
}

#[test]
fn main() {
    let world = World::new();

    world
        .component::<Toppings>()
        .bit("bacon", Toppings::BACON)
        .bit("lettuce", Toppings::LETTUCE)
        .bit("tomato", Toppings::TOMATO);

    world.component::<Sandwich>().member::<Toppings>("toppings");

    // Create entity with Sandwich
    let e = world.entity().set(Sandwich {
        toppings: Toppings {
            value: Toppings::BACON | Toppings::LETTUCE,
        },
    });

    // Convert Sandwidth component to flecs expression string
    e.get::<&Sandwich>(|val| {
        println!("{}", world.to_expr(val));
    });

    // Output:
    //  {toppings: lettuce|bacon}
}
