use rusty_tiny_behavior_tree::{BehaviorNodeBase, BehaviorResult, SequenceNode};
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
        BehaviorResult::SUCCESS(vec![arm.name.clone()])
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
                BehaviorResult::SUCCESS(mut s) => {
                    result.append(&mut s);
                    None
                }
                BehaviorResult::FAILURE(f) => return Some(f),
                _ => None,
            }
        };
        if let Some(f) = join_result(&mut self.left_arm_node, &body.left_arm) {
            return BehaviorResult::FAILURE(f);
        }
        if let Some(f) = join_result(&mut self.right_arm_node, &body.right_arm) {
            return BehaviorResult::FAILURE(f);
        }
        BehaviorResult::SUCCESS(result)
    }
}

struct PeelLeftArmNode<T> {
    node: T,
}

impl<'a, T: BehaviorNodeBase<&'a Arm, Vec<String>, ()>> BehaviorNodeBase<&'a Body, Vec<String>, ()>
    for PeelLeftArmNode<T>
{
    fn tick(&mut self, body: &'a Body) -> BResult {
        self.node.tick(&body.left_arm)
    }
}

struct PeelRightArmNode<T> {
    node: T,
}

impl<'a, T: BehaviorNodeBase<&'a Arm, Vec<String>, ()>> BehaviorNodeBase<&'a Body, Vec<String>, ()>
    for PeelRightArmNode<T>
{
    fn tick(&mut self, body: &'a Body) -> BResult {
        self.node.tick(&body.right_arm)
    }
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
        left_arm_node: Box::<dyn BehaviorNodeBase<&Arm, Vec<String>, ()>>::from(Box::new(
            PrintArmNode,
        )),
        right_arm_node: Box::<dyn BehaviorNodeBase<&Arm, Vec<String>, ()>>::from(Box::new(
            PrintArmNode,
        )),
    };
    assert_eq!(
        tree.tick(&body),
        BehaviorResult::SUCCESS(vec!["leftArm".to_owned(), "rightArm".to_owned()])
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

    // fn map_b<'a, T>(node: T) -> Box<dyn BehaviorNodeBase<&'a Body, Vec<String>, ()>>
    //  where T: BehaviorNodeBase<&'a Body, Vec<String>, ()> + 'static
    // {
    //     Box::new(node)
    // }

    let mut tree = SequenceNode::<&Body, Vec<String>, (), _>::new(
        vec![
            Box::<dyn BehaviorNodeBase<&Body, Vec<String>, ()>>::from(Box::new(PeelLeftArmNode {
                node: PrintArmNode,
            })),
            Box::<dyn BehaviorNodeBase<&Body, Vec<String>, ()>>::from(Box::new(PeelRightArmNode {
                node: PrintArmNode,
            })),
        ],
        |last_success: &mut Vec<String>, mut this_success: Vec<String>| {
            last_success.append(&mut this_success)
        },
    );
    assert_eq!(
        tree.tick(&body),
        BehaviorResult::SUCCESS(vec!["leftArm".to_owned(), "rightArm".to_owned()])
    );
}
