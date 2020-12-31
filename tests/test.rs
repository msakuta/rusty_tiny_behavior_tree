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

impl BehaviorNodeBase<(&Arm, &mut Vec<String>)> for PrintArmNode {
    fn tick(&mut self, (arm, result): &mut (&Arm, &mut Vec<String>)) -> BehaviorResult {
        // let (arm, result) = payload;
        result.push(arm.name.clone());
        BehaviorResult::SUCCESS
    }
}

struct PrintBodyNode;

impl BehaviorNodeBase<(&Body, &mut Vec<String>)> for PrintBodyNode {
    fn tick(&mut self, (body, result): &mut (&Body, &mut Vec<String>)) -> BehaviorResult {
        if let BehaviorResult::FAILURE = PrintArmNode.tick(&mut (&body.left_arm, *result)) {
            return BehaviorResult::FAILURE;
        }
        if let BehaviorResult::FAILURE = PrintArmNode.tick(&mut (&body.right_arm, *result)) {
            return BehaviorResult::FAILURE;
        }
        BehaviorResult::SUCCESS
    }
}

struct PeelLeftArmNode<T>{
    node: T,
}

impl<'a, T: BehaviorNodeBase<(&'a Arm, &'a mut Vec<String>)>>
BehaviorNodeBase<(&'a Body, &'a mut Vec<String>)> for PeelLeftArmNode<T>
{
    fn tick(&mut self, (body, result): &mut (&'a Body, &'a mut Vec<String>)) -> BehaviorResult {
        if let BehaviorResult::FAILURE = self.node.tick(&mut (&body.left_arm, result)) {
            return BehaviorResult::FAILURE;
        }
        BehaviorResult::SUCCESS
    }
}


#[test]
fn test_arm() -> Result<(), ()> {
    let body = Body{
        left_arm: Arm{name: "leftArm".to_string()},
        right_arm: Arm{name: "rightArm".to_string()},
    };

    let mut tree = PrintBodyNode;
    let mut result = vec![];
    assert_eq!(tree.tick(&mut (&body, &mut result)), BehaviorResult::SUCCESS);
    assert_eq!(result, vec!["leftArm", "rightArm"]);
    Ok(())
}
