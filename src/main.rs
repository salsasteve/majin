#[cfg(feature = "debug")]
fn main() {
    use majin::core::Unit;         
    use majin::viz::display_trace; 

    let a = Unit::new(2.0);
    let b = Unit::new(3.0);
    let c = a + b;
    display_trace(&c).unwrap();  // Visualize the computation graph in the terminal
}

#[cfg(not(feature = "debug"))]
fn main() {
    println!("Debugging is disabled. Enable the 'debug' feature to run this.");
}