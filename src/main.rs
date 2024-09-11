use majin::core::Op;
use majin::core::Unit;
use ratatui::{
    backend::CrosstermBackend,
    prelude::Rect,
    widgets::{BorderType, Paragraph},
    Terminal,
};

use crossterm::{
    event::EnableMouseCapture, execute, terminal::enable_raw_mode, terminal::EnterAlternateScreen,
};
use std::collections::HashMap;
use std::io;
use tui_nodes::*;

fn main() -> Result<(), io::Error> {
    let root = create_unit_tree();

    let (nodes, edges) = trace(&root);

    let node_metadata = generate_node_metadata(&nodes);

    let node_layouts = create_node_layouts(&nodes, &node_metadata);

    let connections = create_connections(&nodes, &edges);

    print!("\x1b[2J\x1b[1;1H");

    let mut terminal = setup_terminal()?;

    let space = Rect {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
    };
    let mut graph = NodeGraph::new(
        node_layouts,
        connections,
        space.width as usize,
        space.height as usize,
    );

    terminal.draw(|f| {
        graph.calculate();

        let zones = graph.split(space);

        for (idx, ea_zone) in zones.into_iter().enumerate() {
            let label = &node_metadata[idx].0;
            f.render_widget(Paragraph::new(label.clone()), ea_zone);
        }
        f.render_stateful_widget(graph, space, &mut ());
    })?;

    Ok(())
}

fn create_unit_tree() -> Unit {
    let a = Unit::new(2.0f64, "a");
    let b = Unit::new(3.0f64, "b");
    let c = Unit::new(4.0f64, "c");
    let d = Unit::new(5.0f64, "d");
    let e = Unit::new(6.0f64, "e");
    // 25 = (2 + 3) * 4 + 5 + 6
    let mut ab = a + b;
    ab.label = "ab";
    let mut abc = ab * c;
    abc.label = "abc";
    let mut abcd = abc + d;
    abcd.label = "abcd";
    let mut root = abcd + e;
    root.label = "root";

    root
}

fn generate_node_metadata(
    nodes: &Vec<(&Unit, Option<&Unit>, usize)>,
) -> Vec<(String, String, String)> {
    nodes
        .iter()
        .map(|(node, _, _)| {
            let inner_label = match node.op {
                Some(Op::Add(_)) => "+",
                Some(Op::Mul(_)) => "*",
                _ => "?",
            };
            (
                format!("{}:{}", inner_label, node.value),
                inner_label.to_owned(),
                node.label.to_owned(),
            )
        })
        .collect()
}

fn create_node_layouts<'a>(
    nodes: &'a Vec<(&'a Unit, Option<&'a Unit>, usize)>,
    node_metadata: &'a [(String, String, String)],
) -> Vec<NodeLayout<'a>> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, (node, _, _))| {
            let title = &node_metadata[index].2;
            let mut layout = NodeLayout::new((12, 5))
                .with_title(title)
                .with_border_type(BorderType::Rounded);

            match node.op {
                Some(Op::Add(_)) => {
                    layout = layout.with_border_type(BorderType::Thick);
                }
                Some(Op::Mul(_)) => {
                    layout = layout.with_border_type(BorderType::Double);
                }
                _ => {}
            }

            layout
        })
        .collect()
}

fn create_connections(
    nodes: &Vec<(&Unit, Option<&Unit>, usize)>,
    edges: &Vec<(&Unit, &Unit)>,
) -> Vec<Connection> {
    let mut port_usage: HashMap<usize, usize> = HashMap::new();

    edges
        .iter()
        .map(|(from, to)| {
            let from_index = nodes
                .iter()
                .position(|(node, _, _)| *node == *from)
                .unwrap();
            let to_index = nodes.iter().position(|(node, _, _)| *node == *to).unwrap();

            let from_port = *port_usage.entry(from_index).or_insert(0);
            let to_port = *port_usage.entry(to_index).or_insert(0);

            *port_usage.get_mut(&from_index).unwrap() += 1;
            *port_usage.get_mut(&to_index).unwrap() += 1;

            match nodes[to_index].0.op {
                Some(Op::Add(_)) => Connection::new(from_index, from_port, to_index, to_port)
                    .with_line_type(LineType::Thick),
                Some(Op::Mul(_)) => Connection::new(from_index, from_port, to_index, to_port)
                    .with_line_type(LineType::Double),
                _ => Connection::new(from_index, from_port, to_index, to_port)
                    .with_line_type(LineType::Plain),
            }
        })
        .collect()
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn trace(root: &Unit) -> (Vec<(&Unit, Option<&Unit>, usize)>, Vec<(&Unit, &Unit)>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    fn build<'a>(
        v: &'a Unit,
        parent: Option<&'a Unit>,
        nodes: &mut Vec<(&'a Unit, Option<&'a Unit>, usize)>,
        edges: &mut Vec<(&'a Unit, &'a Unit)>,
        level: usize,
    ) {
        // Check if the current node `v` is already in `nodes_with_levels`:
        // - `nodes_with_levels.iter()`: Creates an iterator over the `nodes_with_levels` vector.
        //   Each item in the iterator is a reference to a tuple `(&Unit, usize)`.
        // - `.any(|(node, _)| *node == v)`: The `.any()` method checks if any item in the iterator
        //   satisfies the provided condition. The closure `|(node, _)| *node == v` is the condition.
        //   This closure takes each tuple `(node, level)` (where `level` is ignored with `_`) and checks
        //   if `node` (which is a reference to a `Unit`) matches the node `v`.
        // - `*node`: Dereferences the `&Unit` reference, so you can directly compare it to `v`.
        // - `!`: Negates the result. If `.any()` returns `true` (meaning the node is already in the vector),
        //   the `!` turns it into `false`, indicating that the node should not be added again.
        let node_exists = nodes.iter().any(|(node, _, _)| *node == v);
        if !node_exists {
            nodes.push((v, parent, level));
            for prev in &v.prev {
                edges.push((prev.as_ref(), v));
                build(prev.as_ref(), Some(v), nodes, edges, level + 1);
            }
        }
    }

    build(root, None, &mut nodes, &mut edges, 0);
    (nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_single_node() {
        let root = Unit::new(0.0f64, "root");
        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].0, &root);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_trace_single_node_f32() {
        let root = Unit::new(0.0f64, "root");
        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 1);
        // (&root, parent, level)
        // first element in the vector is the root node
        // the parent is None because it is the root node
        // the level is 0 because it is the root node
        // first element in the tuple is a reference to the root node
        assert_eq!(nodes[0].0, &root);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_trace_multiple_nodes_f32() {
        let leaf1 = Unit::new(2.0f64, "leaf1");
        let leaf2 = Unit::new(3.0f64, "leaf2");
        let root = leaf1.clone() + leaf2.clone();

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 3);
        // search for the reference to the root node
        assert!(nodes.iter().any(|(node, _, _)| *node == &root));
        // search for the reference to the leaf1 node
        assert!(nodes.iter().any(|(node, _, _)| *node == &leaf1));
        // search for the reference to the leaf2 node
        assert!(nodes.iter().any(|(node, _, _)| *node == &leaf2));

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(n1, n2)| *n1 == &leaf1 && *n2 == &root));
        assert!(edges.iter().any(|(n1, n2)| *n1 == &leaf2 && *n2 == &root));
    }

    #[test]
    fn test_trace_multiple_nodes() {
        let leaf1 = Unit::new(2.0f64, "leaf1");
        let leaf2 = Unit::new(3.0f64, "leaf2");
        let root = leaf1.clone() + leaf2.clone();

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().any(|(node, _, _)| *node == &root));
        assert!(nodes.iter().any(|(node, _, _)| *node == &leaf1));
        assert!(nodes.iter().any(|(node, _, _)| *node == &leaf2));

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(n1, n2)| *n1 == &leaf1 && *n2 == &root));
        assert!(edges.iter().any(|(n1, n2)| *n1 == &leaf2 && *n2 == &root));
    }

    #[test]
    fn test_trace_deep_tree() {
        let leaf1 = Unit::new(2.0f64, "leaf1");
        let leaf2 = Unit::new(3.0f64, "leaf2");
        let leaf3 = Unit::new(4.0f64, "leaf3");
        let leaf4 = Unit::new(5.0f64, "leaf4");
        // 25 = (2 + 3) * 4 + 5
        let root = (leaf1.clone() + leaf2.clone()) * leaf3.clone() + leaf4.clone();

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 7);

        // Validate the root node and its connections
        assert!(nodes.iter().any(|(node, _, _)| *node == &root)); // root 25
        assert_eq!(root.value, 25.0);

        // Validate connections and operations
        assert!(nodes.iter().any(|(node, _, _)| **node == *root.prev[0])); // 20 (result of 5 * 4)
        assert_eq!(root.prev[0].value, 20.0);

        assert!(nodes.iter().any(|(node, _, _)| **node == *root.prev[1])); // 5
        assert_eq!(root.prev[1].value, 5.0);

        assert!(nodes
            .iter()
            .any(|(node, _, _)| **node == *root.prev[0].prev[0])); // 5 (result of 2 + 3)
        assert_eq!(root.prev[0].prev[0].value, 5.0);

        assert!(nodes
            .iter()
            .any(|(node, _, _)| **node == *root.prev[0].prev[1])); // 4
        assert_eq!(root.prev[0].prev[1].value, 4.0);

        assert!(nodes
            .iter()
            .any(|(node, _, _)| **node == *root.prev[0].prev[0].prev[0])); // 2
        assert_eq!(root.prev[0].prev[0].prev[0].value, 2.0);

        assert!(nodes
            .iter()
            .any(|(node, _, _)| **node == *root.prev[0].prev[0].prev[1])); // 3
        assert_eq!(root.prev[0].prev[0].prev[1].value, 3.0);

        // Validate the edges
        assert_eq!(edges.len(), 6);
        assert!(edges
            .iter()
            .any(|(n1, n2)| **n1 == *root.prev[0] && **n2 == root)); // 20 -> root
        assert!(edges
            .iter()
            .any(|(n1, n2)| **n1 == *root.prev[1] && **n2 == root)); // 5 -> root

        assert!(edges
            .iter()
            .any(|(n1, n2)| **n1 == *root.prev[0].prev[0] && **n2 == *root.prev[0])); // 5 -> 20
        assert!(edges
            .iter()
            .any(|(n1, n2)| **n1 == *root.prev[0].prev[1] && **n2 == *root.prev[0])); // 4 -> 20

        assert!(
            edges
                .iter()
                .any(|(n1, n2)| **n1 == *root.prev[0].prev[0].prev[0]
                    && **n2 == *root.prev[0].prev[0])
        ); // 2 -> 5
        assert!(
            edges
                .iter()
                .any(|(n1, n2)| **n1 == *root.prev[0].prev[0].prev[1]
                    && **n2 == *root.prev[0].prev[0])
        ); // 3 -> 5
    }
}
