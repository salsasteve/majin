use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::ops::{Add, Mul};
use libm::tanh;

#[derive(Debug, PartialEq, Clone)]
pub struct Unit {
    pub value: f64,
    pub grad: f64,
    pub prev: ArrayVec<Box<Unit>, 2>,
    pub op: Option<Op>,
    pub label: &'static str,
}

impl Unit {
    pub fn new(value: f64, label: &'static str) -> Self {
        Unit {
            value,
            grad: 0.0,
            prev: ArrayVec::new(),
            op: None,
            label,
        }
    }

    pub fn with_child(
        value: f64,
        children: ArrayVec<Unit, 2>,
        op: Op,
        label: &'static str,
    ) -> Self {
        let mut prev = ArrayVec::new();
        for child in children.iter() {
            prev.push(Box::new(child.clone()));
        }
        Unit {
            value,
            grad: 0.0,
            prev,
            op: Some(op),
            label,
        }
    }

    pub fn tanh(&self) -> Self {
        let value = tanh(self.value);
        let mut children = ArrayVec::new();
        children.push(self.clone());
        Unit::with_child(value, children, Op::Tanh('t'), "tanh")
    }

    pub fn backward(&mut self) {
        match self.op {
            Some(Op::Add(_)) => {
                // f(x) = x + y => df/dx = 1, df/dy = 1
                self.prev[0].grad = 1.0 * self.grad;
                self.prev[1].grad = 1.0 * self.grad;
            }
            Some(Op::Mul(_)) => {
                // f(x) = x * y => df/dx = y, df/dy = x
                self.prev[0].grad = self.prev[1].value * self.grad;
                self.prev[1].grad = self.prev[0].value * self.grad;
            }
            Some(Op::Tanh(_)) => {
                // tanh'(x) = 1 - tanh^2(x)
                let t = tanh(self.value);
                self.prev[0].grad = (1.0 - t * t) * self.grad;
            }
            _ => {}
        }
    }

    pub fn traverse_backward(&mut self) {
            self.backward();
            for child in self.prev.iter_mut() {
                child.traverse_backward();
            }
    }

}

impl Add for Unit {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let value = self.value + other.value;
        let mut children = ArrayVec::new();
        children.push(self.clone());
        children.push(other.clone());
        Unit::with_child(value, children, Op::Add('+'), "result")
    }
}

impl Mul for Unit {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let value = self.value * other.value;
        let mut children = ArrayVec::new();
        children.push(self.clone());
        children.push(other.clone());
        Unit::with_child(value, children, Op::Mul('*'), "result")
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Op {
    Add(char),
    Mul(char),
    Tanh(char),
    Sigmoid(char),
    Relu(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let a = Unit::new(5.0f64, "a");
        let b = Unit::new(10.0f64, "b");
        let mut result = a.clone() + b.clone();
        result.label = "result";
        let mut children = ArrayVec::new();
        children.push(a);
        children.push(b);
        let ans = Unit::with_child(15f64, children, Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_multiplication() {
        let a = Unit::new(3.0f64, "a");
        let b = Unit::new(4.0f64, "b");
        let mut result = a.clone() * b.clone();
        result.label = "result";
        let mut children = ArrayVec::new();
        children.push(a);
        children.push(b);
        let ans = Unit::with_child(12.0f64, children, Op::Mul('*'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_all() {
        let a = Unit::new(2.0f64, "a");
        let b = Unit::new(-3.0f64, "b");
        let c = Unit::new(10.0f64, "c");
        let mut result = a.clone() * b.clone() + c.clone();
        result.label = "result";
        let mut children = ArrayVec::new();
        children.push(a * b);
        children.push(c);
        let ans = Unit::with_child(4.0f64, children, Op::Add('+'), "result");
        assert_eq!(result, ans);
    }

    #[test]
    fn test_backward() {
        let a = Unit::new(2.0f64, "a");
        let b = Unit::new(-3.0f64, "b");
        let c = Unit::new(10.0f64, "c");
        let intermediate = a * b;
        let mut result = intermediate + c;
        result.label = "result";
        result.grad = 1.0;
        result.backward();
        result.prev[0].backward();
        assert_eq!(result.grad, 1.0);
        assert_eq!(result.prev[0].grad, 1.0);
        assert_eq!(result.prev[1].grad, 1.0);
        assert_eq!(result.prev[0].prev[0].grad, -3.0);
        assert_eq!(result.prev[0].prev[1].grad, 2.0);
    }

    #[test]
    fn test_complex1_backward() {
        let a = Unit::new(2.0f64, "a");
        let b = Unit::new(-3.0f64, "b");
        let c = Unit::new(10.0f64, "c");
        let d = Unit::new(5.0f64, "d");
        let intermediate1 = a * b; // -6
        let intermediate2 = c + d; // 15
        let mut result = intermediate1 * intermediate2;
        result.label = "result";
        result.grad = 1.0;
        result.backward();
        result.prev[0].backward();
        result.prev[1].backward();
        assert_eq!(result.grad, 1.0);
        assert_eq!(result.prev[0].grad, 15.0);
        assert_eq!(result.prev[1].grad, -6.0);
        assert_eq!(result.prev[0].prev[0].grad, -45.0); // 15 * -3
        assert_eq!(result.prev[0].prev[1].grad, 30.0); // 15 * 2
        assert_eq!(result.prev[1].prev[0].grad, -6.0); // -6 * 1
        assert_eq!(result.prev[1].prev[1].grad, -6.0); // -6 * 1
    }

    #[test]
    fn test_complex2_backward() {
        let tolerance = 1e-6;
        let a = Unit::new(0.50f64, "a");
        let b = Unit::new(0.75f64, "b");
        let c = Unit::new(0.25f64, "c");
        let d = Unit::new(0.10f64, "d");
        let intermediate1 = a * b; // 0.375
        let intermediate2 = c + d; // 0.35
        let intermediate3 = intermediate1 * intermediate2; // 0.13125
        let mut result = intermediate3.tanh(); // 0.1305
        result.label = "result";
        result.grad = 1.0;
        result.backward(); //  (1 - tanh^2(0.1305)) * 1 = 0.999993
        assert!((result.prev[0].value - 0.13125).abs() < tolerance);
        assert_eq!(result.grad, 1.0);
        assert!((result.prev[0].grad - 0.983161).abs() < tolerance);
        result.prev[0].backward();
        assert_eq!(result.prev[0].prev[0].value, 0.375); // a * b
        assert_eq!(result.prev[0].prev[1].value, 0.35); // c + d
        assert!((result.prev[0].prev[0].grad - 0.344106).abs() < tolerance); // a * b grad
        assert!((result.prev[0].prev[1].grad - 0.368685).abs() < tolerance); // c + d grad
        result.prev[0].prev[0].backward();
        result.prev[0].prev[1].backward();
        assert!((result.prev[0].prev[0].prev[0].grad - 0.25808).abs() < tolerance); // a grad
        assert!((result.prev[0].prev[0].prev[1].grad - 0.172053).abs() < tolerance); // b grad
        assert!((result.prev[0].prev[1].prev[0].grad - 0.368685).abs() < tolerance); // c grad
        assert!((result.prev[0].prev[1].prev[1].grad - 0.368685).abs() < tolerance); // d grad
    }

    #[test]
    fn test_traverse_backward() {
        let tolerance = 1e-6;
        let a = Unit::new(0.50f64, "a");
        let b = Unit::new(0.75f64, "b");
        let c = Unit::new(0.25f64, "c");
        let d = Unit::new(0.10f64, "d");
        let intermediate1 = a * b; // 0.375
        let intermediate2 = c + d; // 0.35
        let intermediate3 = intermediate1 * intermediate2; // 0.13125
        let mut result = intermediate3.tanh(); // 0.1305
        result.label = "result";
        result.grad = 1.0;
        result.traverse_backward();
        
        assert!((result.prev[0].value - 0.13125).abs() < tolerance);
        assert_eq!(result.grad, 1.0);
        assert!((result.prev[0].grad - 0.983161).abs() < tolerance);
        
        assert_eq!(result.prev[0].prev[0].value, 0.375); // a * b
        assert_eq!(result.prev[0].prev[1].value, 0.35);  // c + d
        assert!((result.prev[0].prev[0].grad - 0.344106).abs() < tolerance); // a * b grad
        assert!((result.prev[0].prev[1].grad - 0.368685).abs() < tolerance); // c + d grad
       
        
        assert!((result.prev[0].prev[0].prev[0].grad - 0.25808).abs() < tolerance);  // a grad
        assert!((result.prev[0].prev[0].prev[1].grad - 0.172053).abs() < tolerance); // b grad
        assert!((result.prev[0].prev[1].prev[0].grad - 0.368685).abs() < tolerance); // c grad
        assert!((result.prev[0].prev[1].prev[1].grad - 0.368685).abs() < tolerance); // d grad
    }

    #[test]
    fn test_operation_enum() {
        let add_op = Op::Add('+');
        let mul_op = Op::Mul('*');

        assert_eq!(add_op, Op::Add('+'));
        assert_eq!(mul_op, Op::Mul('*'));
    }
}
