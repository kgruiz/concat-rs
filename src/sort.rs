use std::cmp::Ordering;
use std::path::Path;

pub fn version_path_cmp(a: &Path, b: &Path) -> Ordering {
    version_str_cmp(&a.to_string_lossy(), &b.to_string_lossy())
}

fn version_str_cmp(a: &str, b: &str) -> Ordering {
    let mut a_iter = Segments::new(a);
    let mut b_iter = Segments::new(b);

    loop {
        match (a_iter.next(), b_iter.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(Segment::Text(a_seg)), Some(Segment::Text(b_seg))) => {
                let ord = a_seg.cmp(b_seg);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(Segment::Number(a_num)), Some(Segment::Number(b_num))) => {
                let ord = a_num.cmp(&b_num);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(Segment::Text(_)), Some(Segment::Number(_))) => return Ordering::Less,
            (Some(Segment::Number(_)), Some(Segment::Text(_))) => return Ordering::Greater,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Segment<'a> {
    Text(&'a str),
    Number(u128),
}

struct Segments<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Segments<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn next(&mut self) -> Option<Segment<'a>> {
        if self.pos >= self.input.len() {
            return None;
        }

        let bytes = self.input.as_bytes();
        let start = self.pos;
        let is_digit = bytes[start].is_ascii_digit();

        self.pos += 1;
        while self.pos < bytes.len() && bytes[self.pos].is_ascii_digit() == is_digit {
            self.pos += 1;
        }

        let slice = &self.input[start..self.pos];

        if is_digit {
            let slice = slice.trim_start_matches('0');
            let parsed = slice.parse::<u128>().unwrap_or(0);
            return Some(Segment::Number(parsed));
        }

        Some(Segment::Text(slice))
    }
}
