use enumflags2::{bitflags, BitFlag, BitFlags};

type Point = (f32, f32);

struct Rectangle {
    min: Point,
    max: Point,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum OutCode {
    Left = 0b0001,
    Right = 0b0010,
    Bottom = 0b0100,
    Top = 0b1000,
}

const INSIDE: BitFlags<OutCode> = OutCode::empty();

impl Rectangle {
    fn out_code(&self, point: Point) -> BitFlags<OutCode> {
        let mut code = INSIDE;
        if point.0 < self.min.0 {
            code |= OutCode::Left;
        } else if point.0 > self.max.0 {
            code |= OutCode::Right;
        }
        if point.1 < self.min.1 {
            code |= OutCode::Bottom;
        } else if point.1 > self.max.1 {
            code |= OutCode::Top;
        }
        code
    }

    // Cohen-Sutherland, based on the wikipedia implementation
    // https://en.wikipedia.org/wiki/Cohen%E2%80%93Sutherland_algorithm
    /// Clips a line to a rectangle.
    /// Returns the clipped line, and whether the line is visible.
    fn line_clip(&self, mut line: (Point, Point)) -> (Point, Point, bool) {
        let mut code_a = self.out_code(line.0);
        let mut code_b = self.out_code(line.1);

        let mut accept = false;

        loop {
            if (code_a | code_b).is_empty() {
                // bitwise OR is 0: both points inside window; trivially accept and exit loop
                accept = true;
                break;
            } else if !(code_a & code_b).is_empty() {
                // bitwise AND is not 0: both points share an outside zone (LEFT, RIGHT, TOP,
                // or BOTTOM), so both must be outside window; exit loop (accept is false)
                break;
            } else {
                // At least one endpoint is outside the clip rectangle; pick it.
                let is_code_out_a = !code_a.is_empty();
                let code_out = if is_code_out_a { code_a } else { code_b };

                let ((x0, y0), (x1, y1)) = line;
                let Rectangle {
                    min: (xmin, ymin),
                    max: (xmax, ymax),
                } = *self;

                // failed both tests, so calculate the line segment to clip
                // from an outside point to an intersection with clip edge

                // Now find the intersection point;
                // use formulas:
                //   slope = (y1 - y0) / (x1 - x0)
                //   x = x0 + (1 / slope) * (ym - y0), where ym is ymin or ymax
                //   y = y0 + slope * (xm - x0), where xm is xmin or xmax
                // No need to worry about divide-by-zero because, in each case, the
                // outcode bit being tested guarantees the denominator is non-zero
                let (x, y) = match code_out {
                    x if x.contains(OutCode::Top) => {
                        (x0 + (x1 - x0) * (ymax - y0) / (y1 - y0), ymax)
                    }
                    x if x.contains(OutCode::Bottom) => {
                        (x0 + (x1 - x0) * (ymin - y0) / (y1 - y0), ymin)
                    }
                    x if x.contains(OutCode::Right) => {
                        (xmax, y0 + (y1 - y0) * (xmax - x0) / (x1 - x0))
                    }
                    x if x.contains(OutCode::Left) => {
                        (xmin, y0 + (y1 - y0) * (xmin - x0) / (x1 - x0))
                    }
                    _ => unreachable!("Non-empty code_out must contain one of the above"),
                };

                // Now we move outside point to intersection point to clip
                // and get ready for next pass.
                if is_code_out_a {
                    line.0 = (x, y);
                    code_a = self.out_code(line.0);
                } else {
                    line.1 = (x, y);
                    code_b = self.out_code(line.1);
                }
            }
        }
        return (line.0, line.1, accept);
    }
}

// impl<Iter: Iterator<Item = Point>> LineClipIter<Iter> {
//     fn contains(&self, x: f32, y: f32) -> bool {
//         self.bounds..contains(x) && self.bounds.1.contains(y)
//     }
// }

// iter: 0 1  2  3
// last:   0  1  2  3

struct LineClipIter<Iter: Iterator<Item = Point>> {
    iter: Iter,
    next_return: Option<Point>,
    last: Option<Point>,
    current: Option<Point>,
    bounds: Rectangle,
}

impl<Iter: Iterator<Item = Point>> LineClipIter<Iter> {
    fn step(&mut self) {
        self.last = self.current;
        self.current = self.iter.next();
    }
}

impl<Iter: Iterator<Item = Point>> Iterator for LineClipIter<Iter> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.last.zip(self.current) {
            let (a, b, accept) = self.bounds.line_clip(line);
            if accept {
                self.next_return = Some(b);
                self.step();
                return Some(a);
            }
        }

        self.lhs = !self.lhs;
    }
}
