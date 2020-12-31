use std::cmp::PartialEq;

#[derive(PartialEq, Debug)]
pub enum BehaviorResult {
    IDLE,
    RUNNING,
    SUCCESS,
    FAILURE
}


pub trait BehaviorNodeBase<Payload>{
    fn tick(&mut self, payload: &mut Payload) -> BehaviorResult;
    fn set_parent(&mut self, parent: &dyn BehaviorNodeBase<Payload>){
    }
}

pub struct SequenceNode<Payload> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload>>>,
}

impl<Payload> BehaviorNodeBase<Payload> for SequenceNode<Payload> {
    fn tick(&mut self, payload: &mut Payload) -> BehaviorResult {
        for node in &mut self.children {
            match node.tick(payload) {
                BehaviorResult::FAILURE => return BehaviorResult::FAILURE,
                _ => (),
            }
        }
        BehaviorResult::SUCCESS
    }
}

pub struct FallbackNode<Payload> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload>>>,
}

impl<Payload> BehaviorNodeBase<Payload> for FallbackNode<Payload> {
    fn tick(&mut self, payload: &mut Payload) -> BehaviorResult {
        for node in &mut self.children {
            if let BehaviorResult::SUCCESS = node.tick(payload) {
                return BehaviorResult::SUCCESS;
            }
        }
        BehaviorResult::FAILURE
    }
}
