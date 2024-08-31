use std::collections::HashSet;
use std::fmt::{self, Write, Debug};
use std::string::String;
use crate::core::Unit;

pub fn trace<T>(root: &Unit<T>) -> (HashSet<*const Unit<T>>, HashSet<(*const Unit<T>, *const Unit<T>)>) {
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();

    fn build<T>(v: &Unit<T>, nodes: &mut HashSet<*const Unit<T>>, edges: &mut HashSet<(*const Unit<T>, *const Unit<T>)>) {
        if nodes.insert(v as *const _) {
            for child in &v.child {
                edges.insert((child.as_ref() as *const _, v as *const _));
                build(child.as_ref(), nodes, edges);
            }
        }
    }

    build(root, &mut nodes, &mut edges);
    (nodes, edges)
}

pub fn draw_ascii<T: Debug>(root: &Unit<T>) -> String {
    let (nodes, edges) = trace(root);
    let mut output = String::new();

    for &node in &nodes {
        let unit = unsafe { &*node };
        writeln!(output, "Node {}: Value: {:?}", node as usize, unit.value).unwrap();
    }

    writeln!(output, "\nEdges:").unwrap();

    for &(parent, child) in &edges {
        writeln!(output, "{} -> {}", parent as usize, child as usize).unwrap();
    }

    output
}

pub fn display_trace<T: Debug>(root: &Unit<T>) -> Result<(), fmt::Error> {
    let output = draw_ascii(root);
    println!("{}", output);  // Print the ASCII graph to the terminal
    Ok(())
}