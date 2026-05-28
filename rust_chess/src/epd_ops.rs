use std::collections::HashMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::board::Board;
use crate::py_move::PyMove;

#[derive(Clone, Debug)]
pub enum EpdOperand {
    None,
    String(String),
    Integer(i64),
    Float(f64),
    Move(PyMove),
    MoveList(Vec<PyMove>),
}

pub type EpdOperations = HashMap<String, EpdOperand>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Opcode,
    AfterOpcode,
    Numeric,
    String,
    StringEscape,
    San,
}

fn is_epd_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n')
}

pub fn split_epd_fields(epd: &str) -> Vec<&str> {
    let trimmed = epd.trim().trim_end_matches(';');
    let mut parts = Vec::new();
    let mut remainder = trimmed;

    for _ in 0..4 {
        remainder = remainder.trim_start_matches(is_epd_whitespace);
        if remainder.is_empty() {
            return parts;
        }

        let end = remainder.find(is_epd_whitespace).unwrap_or(remainder.len());
        parts.push(&remainder[..end]);
        remainder = &remainder[end..];
    }

    remainder = remainder.trim_start_matches(is_epd_whitespace);
    if !remainder.is_empty() {
        parts.push(remainder);
    }

    parts
}

pub fn validate_epd_opcode(opcode: &str) -> PyResult<()> {
    if opcode.is_empty() {
        return Err(PyValueError::new_err(
            "empty string is not a valid epd opcode",
        ));
    }
    if opcode == "-" {
        return Err(PyValueError::new_err("dash (-) is not a valid epd opcode"));
    }

    let mut chars = opcode.chars();
    let first = chars
        .next()
        .ok_or_else(|| PyValueError::new_err("empty string is not a valid epd opcode"))?;
    if !first.is_alphabetic() {
        return Err(PyValueError::new_err(format!(
            "expected epd opcode to start with a letter, got: {opcode:?}"
        )));
    }

    for blacklisted in [' ', '\n', '\t', '\r'] {
        if opcode.contains(blacklisted) {
            return Err(PyValueError::new_err(format!(
                "invalid character {blacklisted:?} in epd opcode: {opcode:?}"
            )));
        }
    }

    Ok(())
}

fn is_move_list_opcode(opcode: &str) -> bool {
    matches!(opcode, "pv" | "am" | "bm")
}

fn set_none_operation(operations: &mut EpdOperations, opcode: &str) {
    operations.insert(
        opcode.to_owned(),
        if is_move_list_opcode(opcode) {
            EpdOperand::MoveList(Vec::new())
        } else {
            EpdOperand::None
        },
    );
}

fn parse_numeric_operand(opcode: &str, operand: &str) -> PyResult<EpdOperand> {
    if operand.contains('.') || operand.contains('e') || operand.contains('E') {
        let parsed: f64 = operand.parse().map_err(|_| {
            PyValueError::new_err(format!(
                "invalid numeric operand for epd operation {opcode:?}: {operand:?}"
            ))
        })?;
        if !parsed.is_finite() {
            return Err(PyValueError::new_err(format!(
                "invalid numeric operand for epd operation {opcode:?}: {operand:?}"
            )));
        }
        Ok(EpdOperand::Float(parsed))
    } else {
        let parsed: i64 = operand.parse().map_err(|_| {
            PyValueError::new_err(format!(
                "invalid numeric operand for epd operation {opcode:?}: {operand:?}"
            ))
        })?;
        Ok(EpdOperand::Integer(parsed))
    }
}

fn parse_san_operand(
    board: &Bound<'_, Board>,
    opcode: &str,
    operand: &str,
) -> PyResult<EpdOperand> {
    if opcode == "pv" {
        let kwargs = PyDict::new(board.py());
        kwargs.set_item("stack", false)?;
        let position_any = board.call_method("copy", (), Some(&kwargs))?;
        let position = position_any.cast_into::<Board>()?;
        let mut variation = Vec::new();

        for token in operand.split_whitespace() {
            let move_obj: PyMove = position.call_method1("parse_xboard", (token,))?.extract()?;
            variation.push(move_obj.clone());
            position.call_method1("push", (move_obj,))?;
        }

        Ok(EpdOperand::MoveList(variation))
    } else if matches!(opcode, "bm" | "am") {
        let mut moves = Vec::new();
        for token in operand.split_whitespace() {
            moves.push(board.call_method1("parse_xboard", (token,))?.extract()?);
        }
        Ok(EpdOperand::MoveList(moves))
    } else {
        Ok(EpdOperand::Move(
            board.call_method1("parse_xboard", (operand,))?.extract()?,
        ))
    }
}

pub fn parse_epd_ops(board: &Bound<'_, Board>, operation_part: &str) -> PyResult<EpdOperations> {
    let mut operations = HashMap::new();
    let mut state = ParseState::Opcode;
    let mut opcode = String::new();
    let mut operand = String::new();

    for ch in operation_part
        .chars()
        .map(Some)
        .chain(std::iter::once(None))
    {
        match state {
            ParseState::Opcode => {
                if ch.is_some_and(is_epd_whitespace) {
                    if opcode == "-" {
                        opcode.clear();
                    } else if !opcode.is_empty() {
                        validate_epd_opcode(&opcode)?;
                        state = ParseState::AfterOpcode;
                    }
                } else if ch.is_none() || ch == Some(';') {
                    if opcode == "-" {
                        opcode.clear();
                    } else if !opcode.is_empty() {
                        set_none_operation(&mut operations, &opcode);
                        opcode.clear();
                    }
                } else if let Some(ch) = ch {
                    opcode.push(ch);
                }
            }
            ParseState::AfterOpcode => {
                if ch.is_some_and(is_epd_whitespace) {
                } else if ch == Some('"') {
                    state = ParseState::String;
                } else if ch.is_none() || ch == Some(';') {
                    if !opcode.is_empty() {
                        set_none_operation(&mut operations, &opcode);
                        opcode.clear();
                    }
                    state = ParseState::Opcode;
                } else if ch.is_some_and(|c| matches!(c, '+' | '-' | '.' | '0'..='9')) {
                    operand.push(ch.expect("checked is_some"));
                    state = ParseState::Numeric;
                } else if let Some(ch) = ch {
                    operand.push(ch);
                    state = ParseState::San;
                }
            }
            ParseState::Numeric => {
                if ch.is_none() || ch == Some(';') {
                    let value = parse_numeric_operand(&opcode, &operand)?;
                    operations.insert(opcode.clone(), value);
                    opcode.clear();
                    operand.clear();
                    state = ParseState::Opcode;
                } else if let Some(ch) = ch {
                    operand.push(ch);
                }
            }
            ParseState::String => {
                if ch.is_none() || ch == Some('"') {
                    operations.insert(opcode.clone(), EpdOperand::String(operand.clone()));
                    opcode.clear();
                    operand.clear();
                    state = ParseState::Opcode;
                } else if ch == Some('\\') {
                    state = ParseState::StringEscape;
                } else if let Some(ch) = ch {
                    operand.push(ch);
                }
            }
            ParseState::StringEscape => match ch {
                None => {
                    operations.insert(opcode.clone(), EpdOperand::String(operand.clone()));
                    opcode.clear();
                    operand.clear();
                    state = ParseState::Opcode;
                }
                Some('r') => {
                    operand.push('\r');
                    state = ParseState::String;
                }
                Some('n') => {
                    operand.push('\n');
                    state = ParseState::String;
                }
                Some('t') => {
                    operand.push('\t');
                    state = ParseState::String;
                }
                Some(ch) => {
                    operand.push(ch);
                    state = ParseState::String;
                }
            },
            ParseState::San => {
                if ch.is_none() || ch == Some(';') {
                    let value = parse_san_operand(board, &opcode, &operand)?;
                    operations.insert(opcode.clone(), value);
                    opcode.clear();
                    operand.clear();
                    state = ParseState::Opcode;
                } else if let Some(ch) = ch {
                    operand.push(ch);
                }
            }
        }
    }

    Ok(operations)
}

pub fn hmvc(operations: &EpdOperations) -> PyResult<u32> {
    match operations.get("hmvc") {
        Some(EpdOperand::Integer(value)) => (*value)
            .try_into()
            .map_err(|_| PyValueError::new_err(format!("invalid hmvc value: {value}"))),
        Some(EpdOperand::Float(value)) => Err(PyValueError::new_err(format!(
            "invalid hmvc value: {value}"
        ))),
        Some(_) => Err(PyValueError::new_err("invalid hmvc value")),
        None => Ok(0),
    }
}

pub fn fmvn(operations: &EpdOperations) -> PyResult<u32> {
    match operations.get("fmvn") {
        Some(EpdOperand::Integer(value)) => (*value)
            .try_into()
            .map_err(|_| PyValueError::new_err(format!("invalid fmvn value: {value}"))),
        Some(EpdOperand::Float(value)) => Err(PyValueError::new_err(format!(
            "invalid fmvn value: {value}"
        ))),
        Some(_) => Err(PyValueError::new_err("invalid fmvn value")),
        None => Ok(1),
    }
}

fn escape_epd_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

pub fn format_epd_operations(
    board: &Bound<'_, Board>,
    operations: &EpdOperations,
) -> PyResult<String> {
    let mut out = String::new();
    let mut first = true;
    let mut items: Vec<_> = operations.iter().collect();
    items.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (opcode, operand) in items {
        validate_epd_opcode(opcode)?;

        if !first {
            out.push(' ');
        }
        first = false;
        out.push_str(opcode);

        match operand {
            EpdOperand::None => out.push(';'),
            EpdOperand::Move(move_obj) => {
                out.push(' ');
                out.push_str(
                    &board
                        .call_method1("san", (move_obj.clone(),))?
                        .extract::<String>()?,
                );
                out.push(';');
            }
            EpdOperand::Integer(value) => {
                out.push(' ');
                out.push_str(&value.to_string());
                out.push(';');
            }
            EpdOperand::Float(value) => {
                if !value.is_finite() {
                    return Err(PyValueError::new_err(format!(
                        "expected numeric epd operand to be finite, got: {value}"
                    )));
                }
                out.push(' ');
                out.push_str(&value.to_string());
                out.push(';');
            }
            EpdOperand::MoveList(moves) if opcode == "pv" => {
                let kwargs = PyDict::new(board.py());
                kwargs.set_item("stack", false)?;
                let position_any = board.call_method("copy", (), Some(&kwargs))?;
                let position = position_any.cast_into::<Board>()?;
                for move_obj in moves {
                    out.push(' ');
                    out.push_str(
                        &position
                            .call_method1("san", (move_obj.clone(),))?
                            .extract::<String>()?,
                    );
                    position.call_method1("push", (move_obj.clone(),))?;
                }
                out.push(';');
            }
            EpdOperand::MoveList(moves) if matches!(opcode.as_str(), "am" | "bm") => {
                let mut sans = Vec::new();
                for move_obj in moves {
                    sans.push(
                        board
                            .call_method1("san", (move_obj.clone(),))?
                            .extract::<String>()?,
                    );
                }
                sans.sort();
                for san in sans {
                    out.push(' ');
                    out.push_str(&san);
                }
                out.push(';');
            }
            EpdOperand::MoveList(_) => {
                return Err(PyValueError::new_err(format!(
                    "invalid move list operand for epd operation {opcode:?}"
                )));
            }
            EpdOperand::String(value) => {
                out.push(' ');
                out.push('"');
                out.push_str(&escape_epd_string(value));
                out.push('"');
                out.push(';');
            }
        }
    }

    Ok(out)
}

pub fn py_to_epd_operations(
    board: &Bound<'_, Board>,
    operations: Option<&Bound<'_, PyDict>>,
) -> PyResult<EpdOperations> {
    let Some(operations) = operations else {
        return Ok(HashMap::new());
    };

    let mut out = HashMap::new();

    for (opcode_obj, operand) in operations.iter() {
        let opcode: String = opcode_obj.extract()?;
        validate_epd_opcode(&opcode)?;

        let value = if operand.is_none() {
            EpdOperand::None
        } else if let Ok(move_obj) = operand.extract::<PyMove>() {
            EpdOperand::Move(move_obj)
        } else if let Ok(value) = operand.extract::<i64>() {
            EpdOperand::Integer(value)
        } else if let Ok(value) = operand.extract::<f64>() {
            if !value.is_finite() {
                return Err(PyValueError::new_err(format!(
                    "expected numeric epd operand to be finite, got: {value}"
                )));
            }
            EpdOperand::Float(value)
        } else if operand.cast::<PyString>().is_err()
            && matches!(opcode.as_str(), "pv" | "am" | "bm")
        {
            let mut moves = Vec::new();
            for item in operand.try_iter()? {
                moves.push(item?.extract::<PyMove>()?);
            }
            EpdOperand::MoveList(moves)
        } else {
            EpdOperand::String(operand.str()?.to_string())
        };

        out.insert(opcode, value);
    }

    Ok(out)
}

pub fn epd_operations_to_pydict(
    py: Python<'_>,
    operations: &EpdOperations,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    let mut items: Vec<_> = operations.iter().collect();
    items.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (opcode, operand) in items {
        match operand {
            EpdOperand::None => dict.set_item(opcode, py.None())?,
            EpdOperand::String(value) => dict.set_item(opcode, value)?,
            EpdOperand::Integer(value) => dict.set_item(opcode, *value)?,
            EpdOperand::Float(value) => dict.set_item(opcode, *value)?,
            EpdOperand::Move(move_obj) => dict.set_item(opcode, move_obj.clone())?,
            EpdOperand::MoveList(moves) => dict.set_item(opcode, moves.clone())?,
        }
    }

    Ok(dict.unbind())
}
