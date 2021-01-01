use rusty_tiny_behavior_tree::{BehaviorNodeBase, BehaviorResult};
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

struct PrintBodyNode;

impl BehaviorNodeBase<&Body, Vec<String>, ()> for PrintBodyNode {
    fn tick(&mut self, body: &Body) -> BResult {
        let mut result = vec![];
        let mut join_result = |arm| {
            match PrintArmNode.tick(arm) {
                BehaviorResult::SUCCESS(mut s) => {result.append(&mut s); None},
                BehaviorResult::FAILURE(f) => return Some(f),
                _ => None,
            }
        };
        if let Some(f) = join_result(&body.left_arm) {
            return BehaviorResult::FAILURE(f);
        }
        if let Some(f) = join_result(&body.right_arm) {
            return BehaviorResult::FAILURE(f);
        }
        BehaviorResult::SUCCESS(result)
    }
}

struct PeelLeftArmNode<T>{
    node: T,
}

impl<'a, T: BehaviorNodeBase<&'a Arm, Vec<String>, ()>>
BehaviorNodeBase<&'a Body, Vec<String>, ()> for PeelLeftArmNode<T>
{
    fn tick(&mut self, body: &'a Body) -> BResult {
        self.node.tick(&body.left_arm)
    }
}


#[test]
fn test_arm() -> Result<(), ()> {
    let body = Body{
        left_arm: Arm{name: "leftArm".to_string()},
        right_arm: Arm{name: "rightArm".to_string()},
    };

    let mut tree = PrintBodyNode;
    assert_eq!(tree.tick(&body), BehaviorResult::SUCCESS(vec!["leftArm".to_owned(), "rightArm".to_owned()]));
    Ok(())
}
