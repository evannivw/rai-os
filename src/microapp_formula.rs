//! Small arithmetic compiler/interpreter. No allocation, loops in the language,
//! I/O or pointers. Field indices are resolved against earlier numeric columns.
#[derive(Clone, Copy)]
enum Op {
    Literal(i32),
    Field(usize),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
}
pub struct Program {
    ops: [Op; 32],
    len: usize,
}
impl Program {
    fn push(&mut self, op: Op) -> Result<(), &'static str> {
        if self.len == 32 {
            return Err("formula_limit");
        }
        self.ops[self.len] = op;
        self.len += 1;
        Ok(())
    }
    pub fn eval(&self, values: &[Option<i32>; 6]) -> Result<i32, &'static str> {
        let mut stack = [0i32; 32];
        let mut len = 0;
        for op in &self.ops[..self.len] {
            match *op {
                Op::Literal(n) => {
                    stack[len] = n;
                    len += 1;
                }
                Op::Field(i) => {
                    stack[len] = values[i].ok_or("formula_missing_input")?;
                    len += 1;
                }
                Op::Neg => {
                    stack[len - 1] = stack[len - 1].checked_neg().ok_or("formula_overflow")?
                }
                _ => {
                    let right = stack[len - 1];
                    let left = stack[len - 2];
                    len -= 1;
                    stack[len - 1] = match op {
                        Op::Add => left.checked_add(right),
                        Op::Sub => left.checked_sub(right),
                        Op::Mul => left.checked_mul(right),
                        Op::Div => {
                            if right == 0 {
                                return Err("formula_div_zero");
                            }
                            left.checked_div(right)
                        }
                        _ => unreachable!(),
                    }
                    .ok_or("formula_overflow")?;
                }
            }
        }
        // Only the compiler constructs programs, guaranteeing stack shape.
        Ok(stack[0])
    }
}
struct Parser<'a, F> {
    bytes: &'a [u8],
    pos: usize,
    lookup: F,
    program: Program,
}
impl<F: Fn(&str) -> Option<usize>> Parser<'_, F> {
    fn peek(&mut self) -> Option<u8> {
        while self.bytes.get(self.pos) == Some(&b' ') {
            self.pos += 1;
        }
        self.bytes.get(self.pos).copied()
    }
    fn expr(&mut self, depth: usize) -> Result<(), &'static str> {
        self.term(depth)?;
        while let Some(op @ (b'+' | b'-')) = self.peek() {
            self.pos += 1;
            self.term(depth)?;
            self.program
                .push(if op == b'+' { Op::Add } else { Op::Sub })?;
        }
        Ok(())
    }
    fn term(&mut self, depth: usize) -> Result<(), &'static str> {
        self.atom(depth)?;
        while let Some(op @ (b'*' | b'/')) = self.peek() {
            self.pos += 1;
            self.atom(depth)?;
            self.program
                .push(if op == b'*' { Op::Mul } else { Op::Div })?;
        }
        Ok(())
    }
    fn atom(&mut self, depth: usize) -> Result<(), &'static str> {
        if depth >= 8 {
            return Err("formula_limit");
        }
        match self.peek() {
            Some(b'-') => {
                self.pos += 1;
                self.atom(depth + 1)?;
                self.program.push(Op::Neg)
            }
            Some(b'(') => {
                self.pos += 1;
                self.expr(depth + 1)?;
                if self.peek() != Some(b')') {
                    return Err("formula_syntax");
                }
                self.pos += 1;
                Ok(())
            }
            Some(b'$') => {
                self.pos += 1;
                let start = self.pos;
                while self
                    .bytes
                    .get(self.pos)
                    .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
                {
                    self.pos += 1;
                }
                if self.pos == start {
                    return Err("formula_syntax");
                }
                let key = core::str::from_utf8(&self.bytes[start..self.pos])
                    .map_err(|_| "formula_syntax")?;
                let index = (self.lookup)(key)
                    .filter(|i| *i < 6)
                    .ok_or("formula_unknown_field")?;
                self.program.push(Op::Field(index))
            }
            Some(b'0'..=b'9') => {
                let mut n = 0i32;
                while let Some(b @ b'0'..=b'9') = self.bytes.get(self.pos) {
                    n = n
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(i32::from(*b - b'0')))
                        .ok_or("formula_overflow")?;
                    self.pos += 1;
                }
                self.program.push(Op::Literal(n))
            }
            _ => Err("formula_syntax"),
        }
    }
}
pub fn compile(
    value: &str,
    lookup: impl Fn(&str) -> Option<usize>,
) -> Result<Program, &'static str> {
    if value.is_empty() || value.len() > 64 {
        return Err("formula_limit");
    }
    let mut p = Parser {
        bytes: value.as_bytes(),
        pos: 0,
        lookup,
        program: Program {
            ops: [Op::Literal(0); 32],
            len: 0,
        },
    };
    p.expr(0)?;
    if p.peek().is_some() {
        return Err("formula_syntax");
    }
    Ok(p.program)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grammar_precedence_limits_and_checked_arithmetic() {
        let lookup = |s: &str| match s {
            "x" => Some(0),
            "y" => Some(1),
            _ => None,
        };
        let mut v = [None; 6];
        v[0] = Some(7);
        v[1] = Some(3);
        for (expr, expected) in [
            ("$x + $y * 2", 13),
            ("($x+$y)*2", 20),
            ("-$x / $y", -2),
            ("$x- -$y", 10),
        ] {
            assert_eq!(compile(expr, lookup).unwrap().eval(&v), Ok(expected));
        }
        assert_eq!(
            compile("1/0", lookup).unwrap().eval(&v),
            Err("formula_div_zero")
        );
        v[0] = Some(i32::MIN);
        v[1] = Some(-1);
        assert_eq!(
            compile("$x/$y", lookup).unwrap().eval(&v),
            Err("formula_overflow")
        );
        for expr in [
            "",
            "$unknown",
            "$x+",
            "1 2",
            "foo()",
            "1;run",
            "(((((((((1)))))))))",
            "2147483648",
            "$x)",
        ] {
            assert!(compile(expr, lookup).is_err(), "{expr}");
        }
        v[0] = None;
        assert_eq!(
            compile("$x", lookup).unwrap().eval(&v),
            Err("formula_missing_input")
        );
    }
}
