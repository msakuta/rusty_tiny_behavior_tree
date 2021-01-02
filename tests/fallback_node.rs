use std::cell::RefCell;
use std::convert::From;
use tiny_behavior_tree::{BehaviorNodeBase, BehaviorResult, FallbackNode};

#[derive(PartialEq, Debug, Clone, Copy)]
struct Door {
    open: bool,
    locked: bool,
}

type RCDoor<'a> = &'a RefCell<Door>;

struct IsDoorOpen;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for IsDoorOpen {
    fn tick(&mut self, door: RCDoor<'a>) -> BehaviorResult<(), ()> {
        let door = door.borrow_mut();
        eprintln!("The door is {}", if door.open { "open" } else { "closed" });
        if door.open {
            BehaviorResult::Success(())
        } else {
            BehaviorResult::Failure(())
        }
    }
}

impl BehaviorNodeBase<&mut Door, (), ()> for IsDoorOpen {
    fn tick(&mut self, door: &mut Door) -> BehaviorResult<(), ()> {
        eprintln!("The door is {}", if door.open { "open" } else { "closed" });
        if door.open {
            BehaviorResult::Success(())
        } else {
            BehaviorResult::Failure(())
        }
    }
}

struct OpenDoor;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for OpenDoor {
    fn tick(&mut self, door: RCDoor) -> BehaviorResult<(), ()> {
        let mut door = door.borrow_mut();
        if !door.locked {
            door.open = true;
            eprintln!("Door opened!");
            BehaviorResult::Success(())
        } else {
            eprintln!("Door was unable to open because it's locked!");
            BehaviorResult::Failure(())
        }
    }
}

#[test]
fn test_opened_door() {
    let door = RefCell::new(Door {
        open: true,
        locked: false,
    });

    let mut tree = FallbackNode::<RCDoor, (), (), _>::new([
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(IsDoorOpen)),
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(OpenDoor)),
    ]);

    assert_eq!(tree.tick(&door), BehaviorResult::Success(()));

    assert_eq!(
        *door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );
}

#[test]
fn test_open_door() {
    let door = RefCell::new(Door {
        open: false,
        locked: false,
    });

    let mut tree = FallbackNode::<RCDoor, (), (), _>::new([
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(IsDoorOpen)),
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(OpenDoor)),
    ]);

    assert_eq!(tree.tick(&door), BehaviorResult::Success(()));

    assert_eq!(
        *door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );
}

#[test]
fn test_open_door_fail() {
    let door = RefCell::new(Door {
        open: false,
        locked: true,
    });

    let mut tree = FallbackNode::<RCDoor, (), (), _>::new([
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(IsDoorOpen)),
        Box::<dyn BehaviorNodeBase<RCDoor, (), ()>>::from(Box::new(OpenDoor)),
    ]);

    assert_eq!(tree.tick(&door), BehaviorResult::Failure(()));

    assert_eq!(
        *door.borrow(),
        Door {
            open: false,
            locked: true
        }
    );
}
