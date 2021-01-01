use std::cmp::PartialEq;

#[derive(PartialEq, Debug)]
pub enum BehaviorResult<R, F> {
    IDLE,
    RUNNING,
    SUCCESS(R),
    FAILURE(F),
}


pub trait BehaviorNodeBase<Payload, R, F>{
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F>;
    fn set_parent(&mut self, parent: &dyn BehaviorNodeBase<Payload, R, F>){
    }
}

pub struct SequenceNode<Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
    merge_result: MR,
}

impl<Payload, R, F, MR> SequenceNode<Payload, R, F, MR> {
    pub fn new<T>(children: T, merge_result: MR) -> Self
        where T: Into<Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>>
    {
        Self{ children: children.into(), merge_result }
    }
}

impl<Payload, R, F, MR> BehaviorNodeBase<Payload, R, F> for SequenceNode<Payload, R, F, MR>
    where R: Default, Payload: Copy + Clone,
        MR: Fn(&mut R, R),
{
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F> {
        let mut last_success = R::default();
        for node in &mut self.children {
            match node.tick(payload) {
                BehaviorResult::SUCCESS(r) => (self.merge_result)(&mut last_success, r),
                BehaviorResult::FAILURE(f) => return BehaviorResult::FAILURE(f),
                _ => (),
            }
        }
        BehaviorResult::SUCCESS(last_success)
    }
}

pub struct FallbackNode<Payload, R, F> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
}

impl<Payload, R, F> BehaviorNodeBase<Payload, R, F> for FallbackNode<Payload, R, F>
    where F: Default, Payload: Copy + Clone
{
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F> {
        let mut last_failure = F::default();
        for node in &mut self.children {
            match node.tick(payload) {
                BehaviorResult::SUCCESS(r) => return BehaviorResult::SUCCESS(r),
                BehaviorResult::FAILURE(f) => last_failure = f,
                _ => (),
            }
        }
        BehaviorResult::FAILURE(last_failure)
    }
}
