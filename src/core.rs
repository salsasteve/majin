use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::ops::{Add, Mul};

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Unit<T> {
    pub value: T,
    // pub grad: f32,
    pub prev: ArrayVec<Box<Unit<T>>, 2>,
    pub op: Option<Op>,
    pub label: &'static str,
    // pub description: Option<[char; 30]>,
}

impl<T> Unit<T> {
    pub fn new(value: T, label: &'static str) -> Self {
        Unit {
            value,
            // grad: 0.0,
            prev: ArrayVec::new(),
            op: None,
            label,
        }
    }

    pub fn with_child(value: T, children: (Unit<T>, Unit<T>), op: Op, label: &'static str) -> Self {
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

impl<T> Add for Unit<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let value = self.value + other.value;
        Unit::with_child(value, (self, other), Op::Add('+'), "result")
    }
}

impl<T> Mul for Unit<T>
where
    T: Mul<Output = T> + Copy,
{
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
    fn test_addition_i32() {
        let a = Unit::new(5i32, "a");
        let b = Unit::new(10i32, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let ans = Unit::with_child(15i32, (a, b), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_addition_f32() {
        let a = Unit::new(5.0f32, "a");
        let b = Unit::new(10.0f32, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let ans = Unit::with_child(15f32, (a, b), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_zero_addition_i8() {
        let a = Unit::new(0i8, "a");
        let b = Unit::new(0i8, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let ans = Unit::with_child(0i8, (a, b), Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_multiplication_i16() {
        let a = Unit::new(3i16, "a");
        let b = Unit::new(4i16, "b");
        let mut result = a.clone() * b.clone();
        result.label = "result";
        let ans = Unit::with_child(12i16, (a, b), Op::Mul('*'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_all_i16() {
        let a = Unit::new(2i16, "a");
        let b = Unit::new(-3i16, "b");
        let c = Unit::new(10i16, "c");
        let mut result = a.clone() * b.clone() + c.clone();
        result.label = "result";
        let ans = Unit::with_child(4i16, (a * b, c), Op::Add('+'), "result");
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
