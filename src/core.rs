use core::ops::{Add, Mul};
use arrayvec::ArrayVec;
use alloc::boxed::Box;

#[derive(Debug, PartialEq, Clone)]
pub struct Unit<T> {
    pub value: T,
    pub child: ArrayVec<Box<Unit<T>>, 10>,
    pub op: Option<Op>, 
}

impl<T> Unit<T> {
    pub fn new(value: T) -> Self {
        Unit { 
            value, 
            child: ArrayVec::new(), 
            op: None 
        }
    }

    pub fn with_child(value: T, child: Unit<T>, op: Op) -> Self {
        let mut children = ArrayVec::new(); 
        children.push(Box::new(child));
        Unit { 
            value, 
            child: children, 
            op: Some(op),
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
        Unit::with_child(value, other, Op::Add('+'))
        
    }
}

impl<T> Mul for Unit<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let value = self.value * other.value;
        Unit::with_child(value, other, Op::Mul('*'))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Add(char),
    Mul(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_i32() {
        let a = Unit::new(5i32);
        let b = Unit::new(10i32);
        let result = a + b.clone();
        let ans = Unit::with_child(15i32, b, Op::Add('+'));
        assert_eq!(result, ans);
    }

    #[test]
    fn test_addition_f32() {
        let a = Unit::new(5.0f32);
        let b = Unit::new(10.0f32);
        let result = a + b.clone();
        let ans = Unit::with_child(15f32, b, Op::Add('+'));
        assert_eq!(result, ans);
    }

    #[test]
    fn test_zero_addition_i8() {
        let a = Unit::new(0i8);
        let b = Unit::new(0i8);
        let result = a + b.clone();
        let ans = Unit::with_child(0i8, b, Op::Add('+'));
        assert_eq!(result, ans);
    }

    #[test]
    fn test_multiplication_i16() {
        let a = Unit::new(3i16);
        let b = Unit::new(4i16);
        let result = a * b.clone();
        let ans = Unit::with_child(12i16, b, Op::Mul('*'));
        assert_eq!(result, ans);
    }

    #[test]
    fn test_all_i16() {
        let a = Unit::new(3i16);
        let b = Unit::new(4i16);
        let c = Unit::new(5i16);
        let result = a * b + c.clone();
        let ans = Unit::with_child(17i16, c, Op::Add('+'));
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