use rusty_tiny_behavior_tree::{peel_node_def, BehaviorNodeBase, BehaviorResult, SequenceNode};
use std::convert::From;

struct Arm {
    name: String,
}

struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

impl<'a> From<&'a Body> for &'a Arm {
    fn from(body: &'a Body) -> &'a Arm {
        &body.left_arm
    }
}

struct PrintArmNode;

type BResult = BehaviorResult<Vec<String>, ()>;

impl BehaviorNodeBase<&Arm, Vec<String>, ()> for PrintArmNode {
    fn tick(&mut self, arm: &Arm) -> BResult {
        BehaviorResult::Success(vec![arm.name.clone()])
    }
}

struct BodyArmsNode<'a> {
    left_arm_node: Box<dyn BehaviorNodeBase<&'a Arm, Vec<String>, ()>>,
    right_arm_node: Box<dyn BehaviorNodeBase<&'a Arm, Vec<String>, ()>>,
}

impl<'a> BehaviorNodeBase<&'a Body, Vec<String>, ()> for BodyArmsNode<'a> {
    fn tick(&mut self, body: &'a Body) -> BResult {
        let mut result = vec![];
        let mut join_result = |node: &mut Box<dyn BehaviorNodeBase<&'a Arm, Vec<String>, ()>>,
                               arm: &'a Arm| {
            match node.tick(arm) {
                BehaviorResult::Success(mut s) => {
                    result.append(&mut s);
                    None
                }
                BehaviorResult::Failure(f) => return Some(f),
                _ => None,
            }
        };
        if let Some(f) = join_result(&mut self.left_arm_node, &body.left_arm) {
            return BehaviorResult::Failure(f);
        }
        if let Some(f) = join_result(&mut self.right_arm_node, &body.right_arm) {
            return BehaviorResult::Failure(f);
        }
        BehaviorResult::Success(result)
    }
}

peel_node_def!(
    PeelLeftArmNode,
    Body,
    Arm,
    Vec<String>,
    (),
    |payload: &'a Body| &payload.left_arm
);
peel_node_def!(
    PeelRightArmNode,
    Body,
    Arm,
    Vec<String>,
    (),
    |payload: &'a Body| &payload.right_arm
);

fn boxify<'a, 'b, T, State>(t: T) -> Box<dyn BehaviorNodeBase<&'a State, Vec<String>, ()> + 'b>
where
    T: BehaviorNodeBase<&'a State, Vec<String>, ()> + 'b,
{
    Box::new(t)
}

#[test]
fn test_arm() -> Result<(), ()> {
    let body = Body {
        left_arm: Arm {
            name: "leftArm".to_string(),
        },
        right_arm: Arm {
            name: "rightArm".to_string(),
        },
    };

    let mut tree = BodyArmsNode {
        left_arm_node: boxify(PrintArmNode),
        right_arm_node: boxify(PrintArmNode),
    };
    assert_eq!(
        tree.tick(&body),
        BehaviorResult::Success(vec!["leftArm".to_owned(), "rightArm".to_owned()])
    );
    Ok(())
}

#[test]
fn test_arm_peel() {
    let body = Body {
        left_arm: Arm {
            name: "leftArm".to_string(),
        },
        right_arm: Arm {
            name: "rightArm".to_string(),
        },
    };

    let mut tree = SequenceNode::<&Body, Vec<String>, (), _>::new_with_merger(
        [
            boxify(PeelLeftArmNode(PrintArmNode)),
            boxify(PeelRightArmNode(PrintArmNode)),
        ],
        |last_success: &mut Vec<String>, mut this_success: Vec<String>| {
            last_success.append(&mut this_success)
        },
    );
    assert_eq!(
        tree.tick(&body),
        BehaviorResult::Success(vec!["leftArm".to_owned(), "rightArm".to_owned()])
    );
}
