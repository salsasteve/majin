use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
#[cfg(feature = "debug")]
use majin::core::Unit;
use ratatui::{backend::CrosstermBackend, widgets::canvas::Canvas, Terminal};
use std::io;

fn main() -> Result<(), io::Error> {
    // let a = Unit::new(2i32);
    // let b = Unit::new(3i32);
    // let c = Unit::new(4i32);
    // let d = Unit::new(5i32);
    // let root = (a + b) * c + d;

    // let (nodes, edges) = trace(&root);

    // print the nodes and edges with lines
    // in the terminal to visualize the tree
    // in this format:
    //     Data 2 ----
    //                 \
    //                  --> Op + --> Data 5 ----
    //                 /                         \
    //     Data 3 ----                            --> Op * --> Data 20 ----
    //                                           /                          \
    //                               Data 4 ----                             --> Data 25
    //                                                                      /
    //                                                         Data  5 ----

    // Coordinates for the triangle shape (caret)
    let x = 10;
    let y = 5;

    // Set up terminal
    let mut terminal = setup_terminal()?;

    // Main loop for rendering the triangle shape
    loop {
        terminal.draw(|f| {
            let size = f.area();
            let canvas = Canvas::default()
                .x_bounds([0.0, size.width as f64])
                .y_bounds([0.0, size.height as f64])
                .paint(|ctx| {
                    // Draw a single triangle shape (caret `^`)
                    ctx.print(x as f64, y as f64, "■");
                });

            f.render_widget(canvas, size);
        })?;

        // Break the loop if 'q' is pressed
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // Restore terminal
    cleanup_terminal(&mut terminal)?;
    Ok(())
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

fn trace<T>(root: &Unit<T>) -> (Vec<&Unit<T>>, Vec<(&Unit<T>, &Unit<T>)>)
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    fn build<'a, T>(
        v: &'a Unit<T>,
        nodes: &mut Vec<&'a Unit<T>>,
        edges: &mut Vec<(&'a Unit<T>, &'a Unit<T>)>,
    ) where
        T: std::fmt::Debug + std::cmp::PartialEq,
    {
        if !nodes.contains(&v) {
            nodes.push(v);
            for prev in &v.prev {
                edges.push((prev.as_ref(), v));
                build(prev.as_ref(), nodes, edges);
            }
        }
    }

    build(root, &mut nodes, &mut edges);
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
