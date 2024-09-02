use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use majin::core::Unit;
use majin::core::Op;
// use ratatui::{backend::CrosstermBackend, widgets::canvas::Canvas, Terminal};
use std::io;
use ratatui::{backend::CrosstermBackend, widgets::{Block, Borders, Paragraph}, layout::{Layout, Constraint, Direction}, style::{Style, Color}, Terminal};

// fn main() -> Result<(), io::Error> {
//     // let a = Unit::new(2i32);
//     // let b = Unit::new(3i32);
//     // let c = Unit::new(4i32);
//     // let d = Unit::new(5i32);
//     // let root = (a + b) * c + d;

//     // let (nodes, edges) = trace(&root);

//     // print the nodes and edges with lines
//     // in the terminal to visualize the tree
//     // in this format:
//     //     Data 2 ----
//     //                 \
//     //                  --> Op + --> Data 5 ----
//     //                 /                         \
//     //     Data 3 ----                            --> Op * --> Data 20 ----
//     //                                           /                          \
//     //                               Data 4 ----                             --> Data 25
//     //                                                                      /
//     //                                                         Data  5 ----

//     // Coordinates for the triangle shape (caret)
//     let x = 10;
//     let y = 5;

//     // Set up terminal
//     let mut terminal = setup_terminal()?;

//     // Main loop for rendering the triangle shape
//     loop {
//         terminal.draw(|f| {
//             let size = f.area();
//             let canvas = Canvas::default()
//                 .x_bounds([0.0, size.width as f64])
//                 .y_bounds([0.0, size.height as f64])
//                 .paint(|ctx| {
//                     // Draw a single triangle shape (caret `^`)
//                     ctx.print(x as f64, y as f64, "■");
//                 });

//             f.render_widget(canvas, size);
//         })?;

//         // Break the loop if 'q' is pressed
//         if let Event::Key(key) = event::read()? {
//             if key.code == KeyCode::Char('q') {
//                 break;
//             }
//         }
//     }

//     // Restore terminal
//     cleanup_terminal(&mut terminal)?;
//     Ok(())
// }

fn main() -> Result<(), io::Error> {
    // Example to visualize
    let a = Unit::new(2i32, Some("a".to_string()));
    let b = Unit::new(3i32, Some("b".to_string()));
    let c = Unit::new(4i32, Some("c".to_string()));
    let d = Unit::new(5i32, Some("d".to_string()));
    let e = Unit::new(6i32, Some("e".to_string()));
    // 25 = (2 + 3) * 4 + 5 + 6
    let mut ab = a + b;
    ab.label = "ab".to_string();
    let mut abc = ab * c;
    abc.label = "abc".to_string();
    let mut abcd = abc + d;
    abcd.label = "abcd".to_string();
    let mut root = abcd + e;
    root.label = "root".to_string();

    // Get the nodes and edges
    let (nodes, edges) = trace(&root);

    // Set up terminal
    let mut terminal = setup_terminal()?;
    let mut scroll: u16 = 0;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let mut graph_text = vec![];

            for (node, parent ,level) in nodes.iter() {
                // Only process nodes that have an operation
                if let Some(op) = &node.op {
                    // Determine indentation based on whether the node is a left or right child
                    let indent = if let Some(parent) = parent {
                        if *parent.prev[0] == **node {
                            level * 4
                        } else {
                            level * 4 + 2
                        }
                    } else {
                        0
                    };

                        

                    let lines = draw_node_with_op(node, op);
                    let width = size.width as usize - indent;
            
                    for line in lines {
                        let centered_line = format!("{:padding$}{}", "", line, padding = 0);
                        graph_text.push(center_text(&centered_line, width));
                    }
                }
                // If the node doesn't have an op, it will be skipped.
            }

            let paragraph = Paragraph::new(graph_text.join("\n"))
                .block(Block::default().borders(Borders::ALL).title("Graph"))
                .style(Style::default().fg(Color::White))
                .scroll((scroll, 0));

            f.render_widget(paragraph, chunks[0]);
        })?;

        // Handle input for scrolling
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if scroll > 0 {
                        scroll -= 1;
                    }
                }
                KeyCode::Down => {
                    scroll += 1;
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    cleanup_terminal(&mut terminal)?;
    Ok(())
}


fn draw_simple_node(node: &Unit<i32>) -> Vec<String> {
    vec![format!("{}", node.value)]
}

fn draw_node_with_op(node: &Unit<i32>, op: &Op) -> Vec<String> {
    let operation_symbol = match op {
        Op::Add(_) => "+",
        Op::Mul(_) => "*",
    };

    let mut lines = Vec::new();

    if node.label == "root" {
        lines.push(format!("{}", node.value));
    }

    lines.push(" |".to_string());
    lines.push(format!(" {}", operation_symbol));
    lines.push(" /   \\".to_string());
    lines.push(format!(" {}   {}", node.prev[0].value, node.prev[1].value));

    lines
}

fn center_text(text: &str, width: usize) -> String {
    let padding = (width.saturating_sub(text.len())) / 2;
    format!("{:padding$}{}", "", text, padding = padding)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn trace<T>(root: &Unit<T>) -> (Vec<(&Unit<T>, Option<&Unit<T>>, usize)>, Vec<(&Unit<T>, &Unit<T>)>)
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    fn build<'a, T>(
        v: &'a Unit<T>,
        parent: Option<&'a Unit<T>>,
        nodes: &mut Vec<(&'a Unit<T>, Option<&'a Unit<T>>, usize)>,
        edges: &mut Vec<(&'a Unit<T>, &'a Unit<T>)>,
        level: usize,
    ) where
        T: std::fmt::Debug + std::cmp::PartialEq,
    {
        // Check if the current node `v` is already in `nodes_with_levels`:
        // - `nodes_with_levels.iter()`: Creates an iterator over the `nodes_with_levels` vector. 
        //   Each item in the iterator is a reference to a tuple `(&Unit<T>, usize)`.
        // - `.any(|(node, _)| *node == v)`: The `.any()` method checks if any item in the iterator 
        //   satisfies the provided condition. The closure `|(node, _)| *node == v` is the condition. 
        //   This closure takes each tuple `(node, level)` (where `level` is ignored with `_`) and checks 
        //   if `node` (which is a reference to a `Unit<T>`) matches the node `v`.
        // - `*node`: Dereferences the `&Unit<T>` reference, so you can directly compare it to `v`.
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
        let root = Unit::new(0i8, Some("root".to_string()));
        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&&root));
        assert!(edges.is_empty());
    }

    #[test]
    fn test_trace_single_node_f32() {
        let root = Unit::new(0.0f32, Some("root".to_string()));
        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&&root));
        assert!(edges.is_empty());
    }

    #[test]
    fn test_trace_multiple_nodes_f32() {
        let leaf1 = Unit::new(2.0f32, Some("leaf1".to_string()));
        let leaf2 = Unit::new(3.0f32, Some("leaf2".to_string()));
        let root = leaf1 + leaf2;

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 3);
        // check for reference to a reference to root
        assert!(nodes.contains(&&root));
        // check for reference to a pointer to leaf1
        assert!(nodes.contains(&&*root.prev[0]));
        // check for reference to a pointer to leaf2
        assert!(nodes.contains(&&*root.prev[1]));

        (edges.len(), 2);
        // check for reference to a tuple containing a reference to leaf1 and a reference to root
        assert!(edges.contains(&(&root.prev[0], &root)));
        // check for reference to a tuple containing a reference to leaf2 and a reference to root
        assert!(edges.contains(&(&root.prev[1], &root)));
    }

    #[test]
    fn test_trace_multiple_nodes() {
        let leaf1 = Unit::new(2i32, Some("leaf1".to_string()));
        let leaf2 = Unit::new(3i32, Some("leaf2".to_string()));
        let root = leaf1 + leaf2;

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 3);
        // check for reference to a reference to root
        assert!(nodes.contains(&&root));
        // check for reference to a pointer to leaf1
        assert!(nodes.contains(&&*root.prev[0]));
        // check for reference to a pointer to leaf2
        assert!(nodes.contains(&&*root.prev[1]));

        (edges.len(), 2);
        // check for reference to a tuple containing a reference to leaf1 and a reference to root
        assert!(edges.contains(&(&root.prev[0], &root)));
        // check for reference to a tuple containing a reference to leaf2 and a reference to root
        assert!(edges.contains(&(&root.prev[1], &root)));
    }

    #[test]
    fn test_trace_deep_tree() {
        let leaf1 = Unit::new(2i32, Some("leaf1".to_string()));
        let leaf2 = Unit::new(3i32, Some("leaf2".to_string()));
        let leaf3 = Unit::new(4i32, Some("leaf3".to_string()));
        let leaf4 = Unit::new(5i32, Some("leaf4".to_string()));
        // 25 = (2 + 3) * 4 + 5
        let root = (leaf1 + leaf2) * leaf3 + leaf4;

        let (nodes, edges) = trace(&root);

        assert_eq!(nodes.len(), 7);

        assert!(nodes.contains(&&root)); // root 25
        assert_eq!(root.value, 25);
        // (20 + 5) = 25
        assert!(nodes.contains(&&*root.prev[0])); // 20
        assert_eq!(root.prev[0].value, 20);
        assert!(nodes.contains(&&*root.prev[1])); // 5
        assert_eq!(root.prev[1].value, 5);
        // (5 * 4) = 20
        assert!(nodes.contains(&&*root.prev[0].prev[0])); // 5
        assert_eq!(root.prev[0].prev[0].value, 5);
        assert!(nodes.contains(&&*root.prev[0].prev[1])); // 4
        assert_eq!(root.prev[0].prev[1].value, 4);
        // (2 + 3) = 5
        assert!(nodes.contains(&&*root.prev[0].prev[0].prev[0])); // 2
        assert_eq!(root.prev[0].prev[0].prev[0].value, 2);
        assert!(nodes.contains(&&*root.prev[0].prev[0].prev[1])); // 3
        assert_eq!(root.prev[0].prev[0].prev[1].value, 3);

        assert_eq!(edges.len(), 6);
        // root = (20 + 5)
        assert!(edges.contains(&(&root.prev[0], &root)));
        assert!(edges.contains(&(&root.prev[1], &root)));
        // 20 = (5 * 4)
        assert!(edges.contains(&(&root.prev[0].prev[0], &root.prev[0])));
        assert!(edges.contains(&(&root.prev[0].prev[1], &root.prev[0])));
        // 5 = (2 + 3)
        assert!(edges.contains(&(&root.prev[0].prev[0].prev[0], &root.prev[0].prev[0])));
        assert!(edges.contains(&(&root.prev[0].prev[0].prev[1], &root.prev[0].prev[0])));
    }
}
