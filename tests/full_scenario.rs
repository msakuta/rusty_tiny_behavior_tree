use tiny_behavior_tree::{
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
    in_room: bool,
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

struct SmashDoor;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for SmashDoor {
    fn tick(&mut self, door: RCDoor) -> BehaviorResult<(), ()> {
        let door = door.borrow();
        if !door.open {
            eprintln!("You smashed the door, but it didn't move a bit.");
            BehaviorResult::Failure(())
        } else {
            eprintln!("Door was already open!");
            BehaviorResult::Failure(())
        }
    }
}

struct HaveKey;

impl<'a> BehaviorNodeBase<RCAgent<'a>, (), ()> for HaveKey {
    fn tick(&mut self, agent: RCAgent<'a>) -> BehaviorResult<(), ()> {
        if agent.borrow().has_key {
            BehaviorResult::Success(())
        } else {
            BehaviorResult::Failure(())
        }
    }
}

struct UnlockDoor;

impl<'a> BehaviorNodeBase<RCDoor<'a>, (), ()> for UnlockDoor {
    fn tick(&mut self, door: RCDoor) -> BehaviorResult<(), ()> {
        let mut door = door.borrow_mut();
        door.locked = false;
        eprintln!("Door unlocked!");
        BehaviorResult::Success(())
    }
}

struct EnterRoom;

impl<'a> BehaviorNodeBase<&'a State, (), ()> for EnterRoom {
    fn tick(&mut self, state: &'a State) -> BehaviorResult<(), ()> {
        let mut agent = state.agent.borrow_mut();
        agent.in_room = true;
        eprintln!("Agent entered the room!");
        BehaviorResult::Success(())
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

    let tree = SequenceNodeRef::<State, (), (), _>::new([
        boxify(FallbackNodeRef::<State, (), (), _>::new([
            boxify(PeelDoorNode(IsDoorOpen)),
            boxify(PeelDoorNode(OpenDoor)),
            boxify(SequenceNodeRef::<State, (), (), _>::new([
                boxify(PeelAgentNode(HaveKey)),
                boxify(PeelDoorNode(UnlockDoor)),
                boxify(PeelDoorNode(OpenDoor)),
            ])),
            boxify(PeelDoorNode(SmashDoor)),
        ])),
        boxify(EnterRoom),
    ]);
    Box::new(tree)
}

#[test]
fn test_unlocked_door() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: false,
        }),
        agent: RefCell::new(Agent {
            has_key: false,
            in_room: false,
        }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::Success(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );

    assert_eq!(
        *state.agent.borrow(),
        Agent {
            has_key: false,
            in_room: true
        }
    );
}

#[test]
fn test_unlock_door() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: true,
        }),
        agent: RefCell::new(Agent {
            has_key: true,
            in_room: false,
        }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::Success(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: true,
            locked: false
        }
    );

    assert_eq!(
        *state.agent.borrow(),
        Agent {
            has_key: true,
            in_room: true
        }
    );
}

#[test]
fn test_unlock_door_fail() {
    let state = State {
        door: RefCell::new(Door {
            open: false,
            locked: true,
        }),
        agent: RefCell::new(Agent {
            has_key: false,
            in_room: false,
        }),
    };

    let mut tree = build_tree();

    assert_eq!(tree.tick(&state), BehaviorResult::Failure(()));

    assert_eq!(
        *state.door.borrow(),
        Door {
            open: false,
            locked: true
        }
    );

    assert_eq!(
        *state.agent.borrow(),
        Agent {
            has_key: false,
            in_room: false
        }
    );
}
