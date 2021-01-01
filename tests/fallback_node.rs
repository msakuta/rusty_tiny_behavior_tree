use rusty_tiny_behavior_tree::{BehaviorNodeBase, BehaviorResult, FallbackNode};
use std::cell::RefCell;
use std::convert::From;
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone, Copy)]
struct Door {
    open: bool,
    locked: bool,
}

struct Agent {
    has_key: bool,
}

struct IsDoorOpen;

impl BehaviorNodeBase<Rc<RefCell<Door>>, (), ()> for IsDoorOpen {
    fn tick(&mut self, door: Rc<RefCell<Door>>) -> BehaviorResult<(), ()> {
        let door = door.borrow_mut();
        eprintln!("The door is {}", if door.open { "open" } else { "closed" });
        if door.open {
            BehaviorResult::SUCCESS(())
        } else {
            BehaviorResult::FAILURE(())
        }
    }
}

impl BehaviorNodeBase<&mut Door, (), ()> for IsDoorOpen {
    fn tick(&mut self, door: &mut Door) -> BehaviorResult<(), ()> {
        eprintln!("The door is {}", if door.open { "open" } else { "closed" });
        if door.open {
            BehaviorResult::SUCCESS(())
        } else {
            BehaviorResult::FAILURE(())
        }
    }
}

struct OpenDoor;

impl BehaviorNodeBase<Rc<RefCell<Door>>, (), ()> for OpenDoor {
    fn tick(&mut self, door: Rc<RefCell<Door>>) -> BehaviorResult<(), ()> {
        let mut door = door.borrow_mut();
        if !door.locked {
            door.open = true;
            eprintln!("Door opened!");
            BehaviorResult::SUCCESS(())
        } else {
            eprintln!("Door was unable to open because it's locked!");
            BehaviorResult::FAILURE(())
        }
    }
}

#[test]
fn test_open_door() {
    let door = Rc::new(RefCell::new(Door {
        open: false,
        locked: false,
    }));

    let mut tree = FallbackNode::<Rc<RefCell<Door>>, (), (), _>::new(
        vec![
            Box::<dyn BehaviorNodeBase<Rc<RefCell<Door>>, (), ()>>::from(Box::new(IsDoorOpen)),
            Box::<dyn BehaviorNodeBase<Rc<RefCell<Door>>, (), ()>>::from(Box::new(OpenDoor)),
        ],
        |_: &mut (), _: ()| (),
    );

    assert_eq!(tree.tick(door.clone()), BehaviorResult::SUCCESS(()));

    assert_eq!(
        *door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );
}
