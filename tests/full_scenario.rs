use rusty_tiny_behavior_tree::{
    BehaviorNodeBase, BehaviorResult, FallbackNodeRef, SequenceNodeRef,
};
use std::cell::RefCell;

#[derive(PartialEq, Debug, Clone, Copy)]
struct Door {
    open: bool,
    locked: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct Agent {
    has_key: bool,
}

type RCDoor<'a> = &'a RefCell<Door>;
type RCAgent<'a> = &'a RefCell<Agent>;

struct State {
    door: RefCell<Door>,
    agent: RefCell<Agent>,
}

struct PeelAgentNode<T>(T);

impl<'a, T: BehaviorNodeBase<RCAgent<'a>, (), ()>> BehaviorNodeBase<&'a State, (), ()>
    for PeelAgentNode<T>
{
    fn tick(&mut self, body: &'a State) -> BehaviorResult<(), ()> {
        self.0.tick(&body.agent)
    }
}

struct PeelDoorNode<T>(T);

impl<'a, T: BehaviorNodeBase<RCDoor<'a>, (), ()>> BehaviorNodeBase<&'a State, (), ()>
    for PeelDoorNode<T>
{
    fn tick(&mut self, body: &'a State) -> BehaviorResult<(), ()> {
        self.0.tick(&body.door)
    }
}

struct IsDoorOpen;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for IsDoorOpen {
    fn tick(&mut self, door: RCDoor<'a>) -> BehaviorResult<(), ()> {
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

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for OpenDoor {
    fn tick(&mut self, door: RCDoor) -> BehaviorResult<(), ()> {
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

struct HaveKey;

impl<'a> BehaviorNodeBase<RCAgent<'a>, (), ()> for HaveKey {
    fn tick(&mut self, agent: RCAgent<'a>) -> BehaviorResult<(), ()> {
        if agent.borrow().has_key {
            BehaviorResult::SUCCESS(())
        } else {
            BehaviorResult::FAILURE(())
        }
    }
}

struct UnlockDoor;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for UnlockDoor {
    fn tick(&mut self, door: RCDoor) -> BehaviorResult<(), ()> {
        let mut door = door.borrow_mut();
        door.locked = false;
        eprintln!("Door unlocked!");
        BehaviorResult::SUCCESS(())
    }
}

fn build_tree<'a, 'b>() -> Box<dyn BehaviorNodeBase<&'a State, (), ()> + 'b>
where
    'a: 'b,
{
    type Node<'a, 'b> = Box<dyn BehaviorNodeBase<&'a State, (), ()> + 'b>;

    fn boxify<'a, 'b, T>(t: T) -> Node<'a, 'b>
    where
        T: BehaviorNodeBase<&'a State, (), ()> + 'b,
    {
        Box::new(t)
    }

    let seqtree = SequenceNodeRef::<State, (), (), _>::new(
        [
            boxify(PeelAgentNode(HaveKey)),
            boxify(PeelDoorNode(UnlockDoor)),
            boxify(PeelDoorNode(OpenDoor)),
        ],
        |_: &mut (), _: ()| (),
    );

    let tree = FallbackNodeRef::<State, (), (), _>::new(
        [
            boxify(PeelDoorNode(IsDoorOpen)),
            boxify(PeelDoorNode(OpenDoor)),
            boxify(seqtree),
        ],
        |_: &mut (), _: ()| (),
    );
    Box::new(tree)
}

#[test]
fn test_unlocked_door() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: false,
        }),
        agent: RefCell::new(Agent { has_key: false }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::SUCCESS(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );

    assert_eq!(*state.agent.borrow(), Agent { has_key: false });
}

#[test]
fn test_unlock_door() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: true,
        }),
        agent: RefCell::new(Agent { has_key: true }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::SUCCESS(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );

    assert_eq!(*state.agent.borrow(), Agent { has_key: true });
}

#[test]
fn test_unlock_door_fail() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: true,
        }),
        agent: RefCell::new(Agent { has_key: false }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::FAILURE(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: false,
            locked: true
        }
    );

    assert_eq!(*state.agent.borrow(), Agent { has_key: false });
}
