//! # Overview
//!
//! This project is an attempt of direct port of [TinyBehaviorTree](https://github.com/msakuta/TinyBehaviorTree),
//! which was in turn inspired by [BehaviorTreeCPP](https://github.com/BehaviorTree/BehaviorTree.CPP.git).
//!
//! The name was converted to snake_case to conform to Rust crate naming convention.
//! ## Motivation
//!
//! For the motivation why we would like to experiment things like this, see [TinyBehaviorTree's README](https://github.com/msakuta/TinyBehaviorTree/blob/master/README.md).
//!
//! The problem with implementation in C++ is that it's not trivial to write deserialization code to convert settings file into a whole behavior tree dynamically.
//! Without deserialization, TinyBehaviorTree is just a fancy way to write
//! function call tree.
//! With Rust's rich reflection and serialization ecosystem (serde),
//! we can hope that it can be achieved much more easily.
//!
//! ## How it looks like
//!
//! The usage is very similar to TinyBehaviorTree.
//!
//!
//! ```rust
//! # use rusty_tiny_behavior_tree::*;
//! // First, you define the state with a data structure.
//! struct Arm {
//!     name: String,
//! }
//!
//! struct Body {
//!     left_arm: Arm,
//!     right_arm: Arm,
//! }
//!
//! let body = Body {
//!     left_arm: Arm {
//!         name: "leftArm".to_string(),
//!     },
//!     right_arm: Arm {
//!         name: "rightArm".to_string(),
//!     },
//! };
//!
//! # peel_node_def!(
//! #     PeelLeftArmNode,
//! #     Body,
//! #     Arm,
//! #     (),
//! #     (),
//! #     |payload: &'a Body| &payload.left_arm
//! # );
//! # peel_node_def!(
//! #     PeelRightArmNode,
//! #     Body,
//! #     Arm,
//! #     (),
//! #     (),
//! #     |payload: &'a Body| &payload.right_arm
//! # );
//! struct PrintArmNode;
//!
//! impl BehaviorNodeBase<&Arm, (), ()> for PrintArmNode {
//!     fn tick(&mut self, arm: &Arm) -> BehaviorResult<(), ()> {
//!         BehaviorResult::Success(())
//!     }
//! }
//! # fn boxify<'a, 'b, T, State>(t: T) -> Box<dyn BehaviorNodeBase<&'a State, (), ()> + 'b>
//! # where
//! #   T: BehaviorNodeBase<&'a State, (), ()> + 'b,
//! #   'a: 'b,
//! # {
//! #   Box::new(t)
//! # }
//!
//! // Then, you define a behavior tree.
//! let mut tree = SequenceNodeRef::<Body, (), (), _>::new(
//!     [
//!         boxify(PeelLeftArmNode(PrintArmNode)),
//!         boxify(PeelRightArmNode(PrintArmNode)),
//!     ],
//!     |_: &mut (), _: ()| (),
//! );
//!
//! // Finally, call `tree.tick()`
//! let result = tree.tick(&body);
//! ```
//!
//! ## How to define your own node
//!
//! The core of the library is the [BehaviorNodeBase] trait.
//! It is generic trait with 3 type parameters, i.e.
//! `BehaviorNodeBase<Payload, R, F>`.
//!
//! * `Payload` is the type that is input into this node.
//! * `R` is the result type that will be returned on success.
//! * `F` is the result type that will be returned on failure.
//!
//! This library uses generic parameters to specify information to pass down to each node's tick() function.
//! Unlike C++, Rust doesn't have variadic templates, so you need to stuff the arguments into a tuple if you want to pass multiple arguments.
//!
//! ```rust
//! # use rusty_tiny_behavior_tree::*;
//! # struct Arm {
//! #     name: String,
//! # }
//!
//! struct PrintArmNode;
//!
//! impl BehaviorNodeBase<&Arm, (), ()> for PrintArmNode {
//!     fn tick(&mut self, arm: &Arm) -> BehaviorResult<(), ()> {
//!         println!("arm: {}", arm.name);
//!         BehaviorResult::Success(())
//!     }
//! }
//! ```
//!
//! The `R` and `F` types work like `Result<T, E>` type in standard library, but this library has its own result type with more variants.
//!
//! ```
//! pub enum BehaviorResult<R, F> {
//!     Idle,
//!     Running,
//!     Success(R),
//!     Failure(F),
//! }
//! ```
//!
//! ## Heterogeneous tree
//!
//! Even if you have nodes with different argument types, you can compose them into a single
//! tree using "Peel" nodes, that will transform data in the parent node into what
//! a child node can accept.
//! We call this node PeelNode because usually child node sees smaller view of parent node's accessible data.
//!
//! Suppose we are developing two-armed robot and want to design a behavior node that processes
//! either one of the arms.
//! The data we are passing to the entire tree is like this
//!
//! ```rust
//! struct Arm {
//!     name: String,
//! }
//!
//! struct Body {
//!     left_arm: Arm,
//!     right_arm: Arm,
//! }
//! ```
//!
//! We can define the node to process an arm:
//!
//! ```rust
//! # use rusty_tiny_behavior_tree::*;
//! # struct Arm {
//! #     name: String,
//! # }
//! # struct Body {
//! #     left_arm: Arm,
//! #     right_arm: Arm,
//! # }
//! struct PrintArmNode;
//!
//! impl BehaviorNodeBase<&Arm, (), ()> for PrintArmNode {
//!     fn tick(&mut self, arm: &Arm) -> BehaviorResult<(), ()> {
//!         println!("arm: {}", arm.name);
//!         BehaviorResult::Success(())
//!     }
//! }
//! ```
//!
//! But the entire tree should accept Body as the argument.
//! How do we do it?
//!
//! The answer is to define PeelNodes for left and right arms.
//! There is a macro [`peel_node_def`](crate::peel_node_def) to simplify this process.
//!
//! ```
//! # use rusty_tiny_behavior_tree::*;
//! # struct Arm {
//! #     name: String,
//! # }
//! # struct Body {
//! #     left_arm: Arm,
//! #     right_arm: Arm,
//! # }
//! peel_node_def!(PeelLeftArmNode, Body, Arm, (), (), |payload: &'a Body| &payload.left_arm);
//! peel_node_def!(PeelRightArmNode, Body, Arm, (), (), |payload: &'a Body| &payload.right_arm);
//! ```
use std::cmp::PartialEq;

/// The result type for behavior nodes.
///
/// It is generic over result type `R` and `F`, which contains success and
/// failure cases' results, respectively.
#[derive(PartialEq, Debug)]
pub enum BehaviorResult<R, F> {
    Idle,
    Running,
    Success(R),
    Failure(F),
}

/// The basis of the behavior tree. Every behavior node implements this trait.
pub trait BehaviorNodeBase<Payload, R, F> {
    fn tick(&mut self, payload: Payload) -> BehaviorResult<R, F>;
}

/// Sequence returns success if all child nodes succeed, otherwise returns failure on first child node's failure.
///
/// It has a handful of generic parameters.
///
/// * `Payload`: the type that is passed down to child nodes
/// * `R`: the result type of success case.
/// * `F`: the result type of failure case.
/// * `MR`: the type of result merger function.
///
/// ## Result merger function
///
/// Sometimes you want to customize how to combine results of multiple child nodes.
/// You can provide result merger function to do so.
///
/// The result merger function is a function-like object trait that has
/// the signature `Fn(&mut R, R)`.
/// The first argument is the existing result type, and the second argument
/// is the result to merge.
///
/// For example, if you want to return a vector of response string,
/// `R` would be `Vec<String>` and `MR` would be `Fn(&mut Vec<String>, Vec<String>)`.
/// And the result merger function would be something like
///
/// ```ignore
/// |result: &mut Vec<String>, mut merge: Vec<String>| result.append(&mut merge)
/// ```
pub struct SequenceNode<Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
    merge_result: MR,
}

impl<Payload, R, F, MR> SequenceNode<Payload, R, F, MR> {
    /// Constructs a [SequenceNode] with children nodes and a merger funtion.
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

/// SequenceNode that takes reference to an argument object.
///
/// You cannot just use `SequenceNode<&YourType, ...>` to pass down a
/// reference to an object, because child nodes will have implicit
/// `dyn BehaviorNodeBase<&'a Payload, R, F> + 'static` trait bound.
/// It means the behavior node has to have static lifetime, which is
/// not easy to achieve when you want to construct the tree dynamically,
/// unless you use mechanisms like lazy_static.
///
/// This node will pass down shared reference, so you cannot mutate the
/// referred object in the child nodes.
/// If you want to do so, use [RefCell] as `Payload`.
pub struct SequenceNodeRef<'a, Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>,
    merge_result: MR,
}

impl<'a, Payload, R, F, MR> SequenceNodeRef<'a, Payload, R, F, MR> {
    /// Constructs a [SequenceNodeRef] with children nodes and merger funtion.
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

/// Fallback returns failure if all child nodes fail, otherwise returns success on first child node's success.
///
/// It has a handful of generic parameters.
///
/// * `Payload`: the type that is passed down to child nodes
/// * `R`: the result type of success case.
/// * `F`: the result type of failure case.
/// * `MR`: the type of result merger function.
///
/// ## Result merger function
///
/// Sometimes you want to customize how to combine results of multiple child nodes.
/// You can provide result merger function to do so.
///
/// The result merger function is a function-like object trait that has
/// the signature `Fn(&mut F, F)`.
/// The first argument is the existing result type, and the second argument
/// is the result to merge.
///
/// For example, if you want to return a vector of response strings,
/// `F` would be `Vec<String>` and `MR` would be `Fn(&mut Vec<String>, Vec<String>)`.
/// And the result merger function would be something like
///
/// ```ignore
/// |result: &mut Vec<String>, mut merge: Vec<String>| result.append(&mut merge)
/// ```
pub struct FallbackNode<Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<Payload, R, F>>>,
    merge_result: MR,
}

impl<Payload, R, F, MR> FallbackNode<Payload, R, F, MR> {
    /// Constructs a [FallbackNode] with children nodes and a merger funtion.
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

/// FallbackNode that takes reference to an argument object.
///
/// You cannot just use `FallbackNode<&YourType, ...>` to pass down a
/// reference to an object, because child nodes will have implicit
/// `dyn BehaviorNodeBase<&'a Payload, R, F> + 'static` trait bound.
/// It means the behavior node has to have static lifetime, which is
/// not easy to achieve when you want to construct the tree dynamically,
/// unless you use mechanisms like lazy_static.
///
/// This node will pass down shared reference, so you cannot mutate the
/// referred object in the child nodes.
/// If you want to do so, use [RefCell] as `Payload`.
pub struct FallbackNodeRef<'a, Payload, R, F, MR> {
    children: Vec<Box<dyn BehaviorNodeBase<&'a Payload, R, F> + 'a>>,
    merge_result: MR,
}

impl<'a, Payload, R, F, MR> FallbackNodeRef<'a, Payload, R, F, MR> {
    /// Constructs a [SequenceNodeRef] with children nodes and merger funtion.
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

/// A utility macro to define "peel nodes".
///
/// # Example
/// ```
/// # #[macro_use]
/// # use rusty_tiny_behavior_tree::peel_node_def;
/// struct Arm {
///     name: String,
/// }
///
/// struct Body {
///     left_arm: Arm,
///     right_arm: Arm,
/// }
///
/// peel_node_def!(PeelLeftArmNode, Body, Arm, (), (), |payload: &'a Body| &payload.left_arm);
/// ```
///
/// # Arguments
///
/// This macro has 6 arguments, which is a lot compared to average macros.
///
/// * name: The name of the newly defined peel node.
/// * parent_payload: The type of the payload given to the parent node.
/// * payload: The type of the payload given to the child node.
/// * r: The type of success result.
/// * f: The type of failure result.
/// * peel: The logic (defined as a lambda expression) to convert a reference to the parent node to child node.
///
/// You need to specify a reference with lifetime `'a` to `peel`
/// lambda expression's argument type.
/// For example, if you specify a type `Payload` to the second argument, it should be `&'a Payload`.
/// This is because the macro expands to a generic impl with <'a>.
///
/// Interestingly, Rust's macros are hygienic about identifier names, but not about lifetimes.
#[macro_export]
macro_rules! peel_node_def {
    ($name:ident, $parent_payload:ty, $payload:ty, $r:ty, $f:ty, $peel:expr) => {
        struct $name<T>(T);

        impl<'a, T: rusty_tiny_behavior_tree::BehaviorNodeBase<&'a $payload, $r, $f>>
            rusty_tiny_behavior_tree::BehaviorNodeBase<&'a $parent_payload, $r, $f> for $name<T>
        {
            fn tick(
                &mut self,
                payload: &'a $parent_payload,
            ) -> rusty_tiny_behavior_tree::BehaviorResult<$r, $f> {
                self.0.tick($peel(payload))
            }
        }
    };
}
