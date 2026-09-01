//! Thread-local storage for CSS `grid-template-areas` and `grid-area` resolution.
//!
//! The `css!` macro generates code that pushes area maps when
//! `grid-template-areas` is applied to a grid container, and child elements
//! with `grid-area: "name"` read the top-of-stack map to compute
//! `grid-column`/`grid-row` placements.

use std::cell::RefCell;

/// RAII guard that pops the grid-areas stack on drop.
pub struct GridAreasGuard {
    _private: (),
}

thread_local! {
    static GRID_AREAS_STACK: RefCell<Vec<Vec<Vec<String>>>> = RefCell::new(Vec::new());
}

/// Push a grid-template-areas map onto the thread-local stack.
/// Returns a guard that pops the stack when dropped (at end of the
/// `css!` closure scope).
#[doc(hidden)]
pub fn __push_grid_areas(rows: Vec<Vec<String>>) -> GridAreasGuard {
    GRID_AREAS_STACK.with(|s| s.borrow_mut().push(rows));
    GridAreasGuard { _private: () }
}

/// Resolve a named grid area to `(col_start, col_end, row_start, row_end)`
/// using 1-indexed grid lines. Returns `None` if the name is not found
/// in the top-of-stack area map.
#[doc(hidden)]
pub fn __resolve_grid_area(name: &str) -> Option<(i16, i16, i16, i16)> {
    GRID_AREAS_STACK.with(|s| {
        let stack = s.borrow();
        let areas = stack.last()?;
        resolve_area(areas, name)
    })
}

fn resolve_area(areas: &[Vec<String>], name: &str) -> Option<(i16, i16, i16, i16)> {
    // Find the bounding box of all cells matching `name`.
    // areas[row][col] = name; row/col are 0-indexed.
    let mut min_row: Option<usize> = None;
    let mut max_row: Option<usize> = None;
    let mut min_col: Option<usize> = None;
    let mut max_col: Option<usize> = None;

    for (r, row) in areas.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if cell == name {
                min_row = Some(min_row.map_or(r, |m| m.min(r)));
                max_row = Some(max_row.map_or(r, |m| m.max(r)));
                min_col = Some(min_col.map_or(c, |m| m.min(c)));
                max_col = Some(max_col.map_or(c, |m| m.max(c)));
            }
        }
    }

    let min_row = min_row?;
    let max_row = max_row?;
    let min_col = min_col?;
    let max_col = max_col?;

    // Convert to 1-indexed grid lines: start = index+1, end = index+2
    // (end is exclusive, pointing to the line after the last cell)
    Some((
        (min_col + 1) as i16,
        (max_col + 2) as i16,
        (min_row + 1) as i16,
        (max_row + 2) as i16,
    ))
}

impl Drop for GridAreasGuard {
    fn drop(&mut self) {
        GRID_AREAS_STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_single_cell() {
        let areas = vec![
            vec!["header".to_string(), "header".to_string()],
            vec!["sidebar".to_string(), "main".to_string()],
        ];
        let r = resolve_area(&areas, "sidebar").unwrap();
        assert_eq!(r, (1, 2, 2, 3)); // col 1-2, row 2-3
    }

    #[test]
    fn resolve_spanning_cell() {
        let areas = vec![
            vec!["header".to_string(), "header".to_string()],
            vec!["sidebar".to_string(), "main".to_string()],
        ];
        let r = resolve_area(&areas, "header").unwrap();
        assert_eq!(r, (1, 3, 1, 2)); // col 1-3, row 1-2
    }

    #[test]
    fn resolve_not_found() {
        let areas = vec![vec!["a".to_string()]];
        assert!(resolve_area(&areas, "b").is_none());
    }

    #[test]
    fn push_pop_stack() {
        let _g1 = __push_grid_areas(vec![vec!["a".to_string()]]);
        assert_eq!(
            __resolve_grid_area("a"),
            Some((1, 2, 1, 2))
        );
        {
            let _g2 = __push_grid_areas(vec![vec!["b".to_string()]]);
            // Inner grid shadows outer
            assert!(__resolve_grid_area("a").is_none());
            assert_eq!(
                __resolve_grid_area("b"),
                Some((1, 2, 1, 2))
            );
        }
        // After inner guard drops, outer is visible again
        assert_eq!(
            __resolve_grid_area("a"),
            Some((1, 2, 1, 2))
        );
    }
}
