use std::{cmp::Ordering, collections::HashMap, ops::Range};

pub fn marked_text_offsets_by(
    marked_text: &str,
    markers: Vec<char>,
) -> (String, HashMap<char, Vec<usize>>) {
    let mut extracted_markers: HashMap<char, Vec<usize>> = Default::default();
    let mut unmarked_text = String::new();

    for char in marked_text.chars() {
        if markers.contains(&char) {
            let char_offsets = extracted_markers.entry(char).or_insert(Vec::new());
            char_offsets.push(unmarked_text.len());
        } else {
            unmarked_text.push(char);
        }
    }

    (unmarked_text, extracted_markers)
}

pub fn marked_text_ranges_by(
    marked_text: &str,
    markers: Vec<TextRangeMarker>,
) -> (String, HashMap<TextRangeMarker, Vec<Range<usize>>>) {
    let all_markers = markers.iter().flat_map(|m| m.markers()).collect();

    let (unmarked_text, mut marker_offsets) = marked_text_offsets_by(marked_text, all_markers);
    let range_lookup = markers
        .into_iter()
        .map(|marker| {
            (
                marker.clone(),
                match marker {
                    TextRangeMarker::Empty(empty_marker_char) => marker_offsets
                        .remove(&empty_marker_char)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|empty_index| empty_index..empty_index)
                        .collect::<Vec<Range<usize>>>(),
                    TextRangeMarker::Range(start_marker, end_marker) => {
                        let starts = marker_offsets.remove(&start_marker).unwrap_or_default();
                        let ends = marker_offsets.remove(&end_marker).unwrap_or_default();
                        assert_eq!(starts.len(), ends.len(), "marked ranges are unbalanced");
                        starts
                            .into_iter()
                            .zip(ends)
                            .map(|(start, end)| {
                                assert!(end >= start, "marked ranges must be disjoint");
                                start..end
                            })
                            .collect::<Vec<Range<usize>>>()
                    }
                    TextRangeMarker::ReverseRange(start_marker, end_marker) => {
                        let starts = marker_offsets.remove(&start_marker).unwrap_or_default();
                        let ends = marker_offsets.remove(&end_marker).unwrap_or_default();
                        assert_eq!(starts.len(), ends.len(), "marked ranges are unbalanced");
                        starts
                            .into_iter()
                            .zip(ends)
                            .map(|(start, end)| {
                                assert!(end >= start, "marked ranges must be disjoint");
                                end..start
                            })
                            .collect::<Vec<Range<usize>>>()
                    }
                },
            )
        })
        .collect();

    (unmarked_text, range_lookup)
}

pub fn marked_text_ranges(input_text: &str, indicate_cursors: bool) -> (String, Vec<Range<usize>>) {
    let mut output_text = String::with_capacity(input_text.len());
    let mut ranges = Vec::new();
    let mut prev_input_ix = 0;
    let mut current_range_start = None;
    let mut current_range_cursor = None;

    for (input_ix, marker) in input_text.match_indices(&['«', '»', 'ˇ']) {
        output_text.push_str(&input_text[prev_input_ix..input_ix]);
        let output_len = output_text.len();
        let len = marker.len();
        prev_input_ix = input_ix + len;

        match marker {
            "ˇ" => {
                if current_range_start.is_some() {
                    if current_range_cursor.is_some() {
                        panic!("duplicate point marker 'ˇ' at index {input_ix}");
                    } else {
                        current_range_cursor = Some(output_len);
                    }
                } else {
                    ranges.push(output_len..output_len);
                }
            }
            "«" => {
                if current_range_start.is_some() {
                    panic!("unexpected range start marker '«' at index {input_ix}");
                }
                current_range_start = Some(output_len);
            }
            "»" => {
                let current_range_start = if let Some(start) = current_range_start.take() {
                    start
                } else {
                    panic!("unexpected range end marker '»' at index {input_ix}");
                };

                let mut reversed = false;
                if let Some(current_range_cursor) = current_range_cursor.take() {
                    if current_range_cursor == current_range_start {
                        reversed = true;
                    } else if current_range_cursor != output_len {
                        panic!("unexpected 'ˇ' marker in the middle of a range");
                    }
                } else if indicate_cursors {
                    panic!("missing 'ˇ' marker to indicate range direction");
                }

                ranges.push(if reversed {
                    output_len..current_range_start
                } else {
                    current_range_start..output_len
                });
            }
            _ => unreachable!(),
        }
    }

    output_text.push_str(&input_text[prev_input_ix..]);
    (output_text, ranges)
}

pub fn marked_text_offsets(marked_text: &str) -> (String, Vec<usize>) {
    let (text, ranges) = marked_text_ranges(marked_text, false);
    (
        text,
        ranges
            .into_iter()
            .map(|range| {
                assert_eq!(range.start, range.end);
                range.start
            })
            .collect(),
    )
}

pub fn generate_marked_text(
    output_text: &str,
    ranges: &[Range<usize>],
    indicate_cursors: bool,
) -> String {
    let mut marked_text = output_text.to_string();
    for range in ranges.iter().rev() {
        if indicate_cursors {
            match range.start.cmp(&range.end) {
                Ordering::Less => {
                    marked_text.insert_str(range.end, "ˇ»");
                    marked_text.insert_str(range.start, "«");
                }
                Ordering::Equal => {
                    marked_text.insert_str(range.start, "ˇ");
                }
                Ordering::Greater => {
                    marked_text.insert_str(range.start, "»");
                    marked_text.insert_str(range.end, "«ˇ");
                }
            }
        } else {
            marked_text.insert_str(range.end, "»");
            marked_text.insert_str(range.start, "«");
        }
    }
    marked_text
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TextRangeMarker {
    Empty(char),
    Range(char, char),
    ReverseRange(char, char),
}

impl TextRangeMarker {
    fn markers(&self) -> Vec<char> {
        match self {
            Self::Empty(m) => vec![*m],
            Self::Range(l, r) => vec![*l, *r],
            Self::ReverseRange(l, r) => vec![*l, *r],
        }
    }
}

impl From<char> for TextRangeMarker {
    fn from(marker: char) -> Self {
        Self::Empty(marker)
    }
}

impl From<(char, char)> for TextRangeMarker {
    fn from((left_marker, right_marker): (char, char)) -> Self {
        Self::Range(left_marker, right_marker)
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_marked_text, marked_text_ranges};

    #[test]
    fn test_marked_text() {
        let (text, ranges) = marked_text_ranges("one «ˇtwo» «threeˇ» «ˇfour» fiveˇ six", true);

        assert_eq!(text, "one two three four five six");
        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0], 7..4);
        assert_eq!(ranges[1], 8..13);
        assert_eq!(ranges[2], 18..14);
        assert_eq!(ranges[3], 23..23);

        assert_eq!(
            generate_marked_text(&text, &ranges, true),
            "one «ˇtwo» «threeˇ» «ˇfour» fiveˇ six"
        );
    }
}
