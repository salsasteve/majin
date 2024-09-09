use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::ops::{Add, Mul};

#[derive(Debug, PartialEq, Clone)]
pub struct Unit {
    pub value: f32,
    // pub grad: f32,
    pub prev: ArrayVec<Box<Unit>, 2>,
    pub op: Option<Op>,
    pub label: &'static str,
    // pub description: Option<[char; 30]>,
}

impl Unit {
    pub fn new(value: f32, label: &'static str) -> Self {
        Unit {
            value,
            // grad: 0.0,
            prev: ArrayVec::new(),
            op: None,
            label,
        }
    }

    pub fn with_child(value: f32, children: (Unit, Unit), op: Op, label: &'static str) -> Self {
        let mut prev = ArrayVec::new();
        prev.push(Box::new(children.0));
        prev.push(Box::new(children.1));
        Unit {
            value,
            // grad: 0.0,
            prev: prev,
            op: Some(op),
            label: label,
        }
    }
}

impl Add for Unit {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let value = self.value + other.value;
        Unit::with_child(value, (self, other), Op::Add('+'), "result")
    }
}

impl Mul for Unit {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let value = self.value * other.value;
        Unit::with_child(value, (self, other), Op::Mul('*'), "result")
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Op {
    Add(char),
    Mul(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let a = Unit::new(5.0f32, "a");
        let b = Unit::new(10.0f32, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let ans = Unit::with_child(15f32, (a, b), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_zero_addition() {
        let a = Unit::new(0.0f32, "a");
        let b = Unit::new(0.0f32, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let ans = Unit::with_child(0f32, (a, b), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_multiplication() {
        let a = Unit::new(3.0f32, "a");
        let b = Unit::new(4.0f32, "b");
        let mut result = a.clone() * b.clone();
        result.label = "result";
        let ans = Unit::with_child(12.0f32, (a, b), Op::Mul('*'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_all() {
        let a = Unit::new(2.0f32, "a");
        let b = Unit::new(-3.0f32, "b");
        let c = Unit::new(10.0f32, "c");
        let mut result = a.clone() * b.clone() + c.clone();
        result.label = "result";
        let ans = Unit::with_child(4.0f32, (a * b, c), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_operation_enum() {
        let add_op = Op::Add('+');
        let mul_op = Op::Mul('*');

        assert_eq!(add_op, Op::Add('+'));
        assert_eq!(mul_op, Op::Mul('*'));
    }
}
