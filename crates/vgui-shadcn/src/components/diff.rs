//! Source-diff and merge-conflict review.
//!
//! A dependency-free line diff built on the classic LCS dynamic program:
//! [`diff_lines`] returns aligned [`DiffLine`]s tagged equal / inserted /
//! deleted with their original line numbers. [`parse_conflicts`] extracts
//! `<<<<<<<` / `=======` / `>>>>>>>` regions. Both feed the viewers:
//! [`diff_viewer`] (unified) and [`DiffEditor`] (a stateful unified/split
//! toggle).

use gpui::{
    div, px, AnyElement, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled,
};
use vgui::{create_signal, next_auto_id, __var_color};

/// How one aligned line changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    /// Present unchanged in both sides (carries both line numbers).
    Equal,
    /// Present only on the new side.
    Insert,
    /// Present only on the old side.
    Delete,
}

/// One aligned row of a diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    /// Change kind.
    pub kind: DiffKind,
    /// 1-based line number on the old side, if present there.
    pub old_no: Option<usize>,
    /// 1-based line number on the new side, if present there.
    pub new_no: Option<usize>,
    /// The line text (without its trailing newline).
    pub text: String,
}

impl DiffLine {
    fn gutter(&self) -> String {
        let old = self.old_no.map(|n| n.to_string()).unwrap_or_default();
        let new = self.new_no.map(|n| n.to_string()).unwrap_or_default();
        format!("{old:>4} {new:<4}")
    }

    fn marker(&self) -> &'static str {
        match self.kind {
            DiffKind::Equal => "  ",
            DiffKind::Insert => "+ ",
            DiffKind::Delete => "- ",
        }
    }
}

/// Compute an LCS-based line diff between `old` and `new`.
pub fn diff_lines(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let n = old.len();
    let m = new.len();

    // LCS length table: lcs[i][j] = LCS of old[i..] and new[j..].
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if old[i] == new[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    // Walk the table to emit an aligned edit script, preferring matches.
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    let (mut old_no, mut new_no) = (1usize, 1usize);
    while i < n && j < m {
        if old[i] == new[j] {
            out.push(DiffLine {
                kind: DiffKind::Equal,
                old_no: Some(old_no),
                new_no: Some(new_no),
                text: old[i].to_string(),
            });
            i += 1;
            j += 1;
            old_no += 1;
            new_no += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            // Delete old[i]: taking the old row yields at least as much LCS.
            out.push(DiffLine {
                kind: DiffKind::Delete,
                old_no: Some(old_no),
                new_no: None,
                text: old[i].to_string(),
            });
            i += 1;
            old_no += 1;
        } else {
            out.push(DiffLine {
                kind: DiffKind::Insert,
                old_no: None,
                new_no: Some(new_no),
                text: new[j].to_string(),
            });
            j += 1;
            new_no += 1;
        }
    }
    while i < n {
        out.push(DiffLine {
            kind: DiffKind::Delete,
            old_no: Some(old_no),
            new_no: None,
            text: old[i].to_string(),
        });
        i += 1;
        old_no += 1;
    }
    while j < m {
        out.push(DiffLine {
            kind: DiffKind::Insert,
            old_no: None,
            new_no: Some(new_no),
            text: new[j].to_string(),
        });
        j += 1;
        new_no += 1;
    }
    out
}

/// One side of a merge conflict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    /// Rows between `<<<<<<<` and `=======` ("ours" / current branch).
    pub ours: Vec<String>,
    /// Rows between `=======` and `>>>>>>>` ("theirs" / incoming branch).
    pub theirs: Vec<String>,
    /// 0-based line index of the opening `<<<<<<<`.
    pub start_line: usize,
}

/// Parse every Git-style conflict region in `text`.
///
/// Recognises `<<<<<<< label`, a single `=======` divider and `>>>>>>> label`.
/// Unterminated regions are ignored (treated as ordinary text).
pub fn parse_conflicts(text: &str) -> Vec<Conflict> {
    let lines: Vec<&str> = text.lines().collect();
    let mut conflicts = Vec::new();
    let mut k = 0;
    while k < lines.len() {
        if lines[k].starts_with("<<<<<<<") {
            let start = k;
            let mut ours = Vec::new();
            let mut p = k + 1;
            while p < lines.len() && !lines[p].starts_with("=======") {
                ours.push(lines[p].to_string());
                p += 1;
            }
            if p >= lines.len() {
                break;
            }
            let mut theirs = Vec::new();
            let mut q = p + 1;
            while q < lines.len() && !lines[q].starts_with(">>>>>>>") {
                theirs.push(lines[q].to_string());
                q += 1;
            }
            if q < lines.len() {
                conflicts.push(Conflict {
                    ours,
                    theirs,
                    start_line: start,
                });
                k = q + 1;
                continue;
            }
        }
        k += 1;
    }
    conflicts
}

fn line_bg(kind: DiffKind) -> gpui::Hsla {
    match kind {
        // Very light red / green tints.
        DiffKind::Delete => gpui::hsla(0.0, 0.7, 0.92, 1.0),
        DiffKind::Insert => gpui::hsla(0.33, 0.5, 0.92, 1.0),
        DiffKind::Equal => gpui::hsla(0., 0., 1., 0.0),
    }
}

fn line_accent(kind: DiffKind) -> gpui::Hsla {
    match kind {
        DiffKind::Delete => gpui::hsla(0.0, 0.65, 0.45, 1.0),
        DiffKind::Insert => gpui::hsla(0.33, 0.5, 0.35, 1.0),
        DiffKind::Equal => __var_color("--muted-foreground", Some(gpui::hsla(0., 0., 0.4, 1.))),
    }
}

fn render_row(line: &DiffLine, split: bool) -> AnyElement {
    let fg = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));
    let bg = line_bg(line.kind);

    let mut row = div()
        .flex()
        .flex_row()
        .w_full()
        .text_size(px(12.5))
        .font_family("monospace")
        .bg(bg);

    if split {
        // Two columns: old on the left, new on the right.
        let (old_text, old_bg) = match line.kind {
            DiffKind::Insert => (String::new(), gpui::hsla(0., 0., 1., 0.)),
            _ => (line.text.clone(), bg),
        };
        let (new_text, new_bg) = match line.kind {
            DiffKind::Delete => (String::new(), gpui::hsla(0., 0., 1., 0.)),
            _ => (line.text.clone(), bg),
        };
        let gutter = line.gutter();
        row = row
            .child(
                div()
                    .w(px(84.))
                    .flex_shrink_0()
                    .text_color(line_accent(line.kind))
                    .child(gutter),
            )
            .child(
                div()
                    .flex_1()
                    .bg(old_bg)
                    .px_2()
                    .text_color(fg)
                    .child(old_text),
            )
            .child(
                div()
                    .flex_1()
                    .bg(new_bg)
                    .px_2()
                    .text_color(fg)
                    .child(new_text),
            );
    } else {
        row = row
            .child(
                div()
                    .w(px(84.))
                    .flex_shrink_0()
                    .px_1()
                    .text_color(line_accent(line.kind))
                    .child(line.gutter()),
            )
            .child(
                div()
                    .w(px(16.))
                    .flex_shrink_0()
                    .text_color(line_accent(line.kind))
                    .font_weight(FontWeight::BOLD)
                    .child(line.marker()),
            )
            .child(div().flex_1().text_color(fg).child(line.text.clone()));
    }
    row.into_any_element()
}

/// Render a unified diff between two texts.
pub fn diff_viewer(old: &str, new: &str) -> AnyElement {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let rows = diff_lines(&old_lines, &new_lines);

    let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));
    let mut body = div().flex().flex_col().w_full();
    for row in &rows {
        body = body.child(render_row(row, false));
    }
    div()
        .w_full()
        .border_1()
        .border_color(border)
        .rounded(px(vgui::__var_number("--radius", Some(6.))))
        .child(body)
        .into_any_element()
}

/// Stateful diff widget with a clickable unified/split mode toggle.
pub struct DiffEditor;

impl DiffEditor {
    /// Render the diff. Clicking the header toggles unified vs split layout.
    pub fn render(old: String, new: String) -> (AnyElement, vgui::ReadSignal<bool>) {
        let (split, set_split) = create_signal(false);
        let is_split = split.get();

        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        let rows = diff_lines(&old_lines, &new_lines);

        let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));
        let header_bg = __var_color("--muted", Some(gpui::hsla(0., 0., 0.95, 1.)));
        let fg = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));

        let read_in_handler = split.clone();
        let toggle = div()
            .id(("diff-mode", next_auto_id()))
            .w(px(96.))
            .px_2()
            .h(px(24.))
            .flex()
            .items_center()
            .text_size(px(12.))
            .text_color(fg)
            .border_1()
            .border_color(border)
            .rounded(px(4.))
            .on_click(move |_ev, _w: &mut gpui::Window, cx: &mut gpui::App| {
                set_split.set(cx, !read_in_handler.get());
            })
            .child(if is_split { "Split: on" } else { "Unified" });

        let mut body = div().flex().flex_col().w_full();
        for row in &rows {
            body = body.child(render_row(row, is_split));
        }

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(30.))
            .px_2()
            .gap_2()
            .bg(header_bg)
            .border_b_1()
            .border_color(border)
            .child(div().flex_1().child("Diff"))
            .child(toggle);
        let el = div()
            .w_full()
            .flex()
            .flex_col()
            .border_1()
            .border_color(border)
            .rounded(px(vgui::__var_number("--radius", Some(6.))))
            .child(header)
            .child(body)
            .into_any_element();
        (el, split)
    }
}

/// Render the merge-conflict regions in `text`: each `<<<<<<< … >>>>>>>`
/// block shows the two sides side by side. An empty column means no conflicts.
pub fn conflict_viewer(text: &str) -> AnyElement {
    let conflicts = parse_conflicts(text);
    let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));
    let fg = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));
    let red = gpui::hsla(0.0, 0.65, 0.45, 1.0);
    let green = gpui::hsla(0.33, 0.5, 0.35, 1.0);

    let mut col = div().flex().flex_col().w_full().gap_2();
    for conflict in &conflicts {
        let mut ours = div()
            .flex_1()
            .border_1()
            .border_color(border)
            .text_size(px(12.5))
            .font_family("monospace")
            .text_color(fg);
        for line in &conflict.ours {
            ours = ours
                .child(div().bg(line_bg(DiffKind::Delete)).text_color(red).child(line.clone()));
        }
        let mut theirs = div()
            .flex_1()
            .border_1()
            .border_color(border)
            .text_size(px(12.5))
            .font_family("monospace")
            .text_color(fg);
        for line in &conflict.theirs {
            theirs = theirs.child(
                div().bg(line_bg(DiffKind::Insert)).text_color(green).child(line.clone()),
            );
        }
        let block = div()
            .flex()
            .flex_row()
            .w_full()
            .gap_2()
            .child(ours)
            .child(theirs);
        col = col.child(block);
    }
    col.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_insertion_and_deletion() {
        let d = diff_lines(&["a", "b", "c"], &["a", "c"]);
        let kinds: Vec<_> = d.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![DiffKind::Equal, DiffKind::Delete, DiffKind::Equal]
        );
        let deleted = &d[1];
        assert_eq!(deleted.text, "b");
        assert_eq!(deleted.old_no, Some(2));
        assert_eq!(deleted.new_no, None);
    }

    #[test]
    fn tracks_both_line_numbers() {
        let d = diff_lines(&["x"], &["x", "y"]);
        assert_eq!(d[0].old_no, Some(1));
        assert_eq!(d[0].new_no, Some(1));
        assert_eq!(d[1].kind, DiffKind::Insert);
        assert_eq!(d[1].new_no, Some(2));
    }

    #[test]
    fn replacement_is_delete_then_insert() {
        let d = diff_lines(&["one"], &["two"]);
        assert_eq!(d[0].kind, DiffKind::Delete);
        assert_eq!(d[1].kind, DiffKind::Insert);
    }

    #[test]
    fn lcs_prefers_longest_common_subsequence() {
        // Moving one line should align the common tail rather than show it
        // as churn: "b" stays equal.
        let d = diff_lines(&["a", "b"], &["b", "c"]);
        let equal: Vec<&str> = d
            .iter()
            .filter(|l| l.kind == DiffKind::Equal)
            .map(|l| l.text.as_str())
            .collect();
        assert_eq!(equal, vec!["b"]);
    }

    #[test]
    fn parses_conflict_region() {
        let text = "keep\n<<<<<<< HEAD\nours line\n=======\ntheirs line\n>>>>>>> branch\nafter";
        let cs = parse_conflicts(text);
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].ours, vec!["ours line".to_string()]);
        assert_eq!(cs[0].theirs, vec!["theirs line".to_string()]);
        assert_eq!(cs[0].start_line, 1);
    }

    #[test]
    fn parses_multiple_conflicts() {
        let text = "<<<<<<<\na\n=======\nb\n>>>>>>>\nmid\n<<<<<<<\nc\n=======\nd\n>>>>>>>";
        assert_eq!(parse_conflicts(text).len(), 2);
    }

    #[test]
    fn unterminated_conflict_is_ignored() {
        let text = "<<<<<<<\na\n=======\nb\n";
        assert!(parse_conflicts(text).is_empty());
    }
}
