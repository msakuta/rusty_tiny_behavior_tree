use std::cmp::PartialEq;

#[derive(PartialEq, Debug)]
pub enum BehaviorResult<R, F> {
    Idle,
    Running,
    Success(R),
    Failure(F),
}

pub trait BehaviorNodeBase<Payload, R, F> {
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F>;
}

pub struct SequenceNode<Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
    merge_result: MR,
}

impl<Payload, R, F, MR> SequenceNode<Payload, R, F, MR> {
    pub fn new<T>(children: T, merge_result: MR) -> Self
    where
        T: Into<Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>>,
    {
        Self {
            children: children.into(),
            merge_result,
        }
    }
}

impl<Payload, R, F, MR> BehaviorNodeBase<Payload, R, F> for SequenceNode<Payload, R, F, MR>
where
    R: Default,
    Payload: Clone,
    MR: Fn(&mut R, R),
{
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F> {
        let mut last_success = R::default();
        for node in &mut self.children {
            match node.tick(payload.clone()) {
                BehaviorResult::Success(r) => (self.merge_result)(&mut last_success, r),
                BehaviorResult::Failure(f) => return BehaviorResult::Failure(f),
                _ => (),
            }
        }
        BehaviorResult::Success(last_success)
    }
}

pub struct SequenceNodeRef<'a, Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>,
    merge_result: MR,
}

impl<'a, Payload, R, F, MR> SequenceNodeRef<'a, Payload, R, F, MR> {
    pub fn new<T>(children: T, merge_result: MR) -> Self
    where
        T: Into<Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>>,
    {
        Self {
            children: children.into(),
            merge_result,
        }
    }
}

impl<'a, Payload, R, F, MR> BehaviorNodeBase<&'a Payload, R, F>
    for SequenceNodeRef<'a, Payload, R, F, MR>
where
    R: Default,
    MR: Fn(&mut R, R),
{
    fn tick(&mut self, payload: &'a Payload) -> BehaviorResult<R, F> {
        let mut last_success = R::default();
        for node in &mut self.children {
            match node.tick(payload) {
                BehaviorResult::Success(r) => (self.merge_result)(&mut last_success, r),
                BehaviorResult::Failure(f) => return BehaviorResult::Failure(f),
                _ => (),
            }
        }
        BehaviorResult::Success(last_success)
    }
}

pub struct FallbackNode<Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
    merge_result: MR,
}

impl<Payload, R, F, MR> FallbackNode<Payload, R, F, MR> {
    pub fn new<T>(children: T, merge_result: MR) -> Self
    where
        T: Into<Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>>,
    {
        Self {
            children: children.into(),
            merge_result,
        }
    }
}

impl<Payload, R, F, MR> BehaviorNodeBase<Payload, R, F> for FallbackNode<Payload, R, F, MR>
where
    F: Default,
    Payload: Clone,
    MR: Fn(&mut F, F),
{
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F> {
        let mut last_failure = F::default();
        for node in &mut self.children {
            match node.tick(payload.clone()) {
                BehaviorResult::Success(r) => return BehaviorResult::Success(r),
                BehaviorResult::Failure(f) => (self.merge_result)(&mut last_failure, f),
                _ => (),
            }
        }
        BehaviorResult::Failure(last_failure)
    }
}

pub struct FallbackNodeRef<'a, Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>,
    merge_result: MR,
}

impl<'a, Payload, R, F, MR> FallbackNodeRef<'a, Payload, R, F, MR> {
    pub fn new<T>(children: T, merge_result: MR) -> Self
    where
        T: Into<Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>>,
    {
        Self {
            children: children.into(),
            merge_result,
        }
    }
}

impl<'a, Payload, R, F, MR> BehaviorNodeBase<&'a Payload, R, F>
    for FallbackNodeRef<'a, Payload, R, F, MR>
where
    F: Default,
    MR: Fn(&mut F, F),
{
    fn tick(&mut self, payload: &'a Payload) -> BehaviorResult<R, F> {
        let mut last_failure = F::default();
        for node in &mut self.children {
            match node.tick(payload) {
                BehaviorResult::Success(r) => return BehaviorResult::Success(r),
                BehaviorResult::Failure(f) => (self.merge_result)(&mut last_failure, f),
                _ => (),
            }
        }
        BehaviorResult::Failure(last_failure)
    }
}
