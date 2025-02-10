/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::process_regexes;
use crate::parse_search_pattern;
use crate::query::QueryTree;
use crate::result::QueryResult;
use crate::QueryError;
use crate::RegexError;

impl std::convert::From<QueryError> for PyErr {
    fn from(err: QueryError) -> PyErr {
        PyValueError::new_err(err.message)
    }
}

impl std::convert::From<RegexError> for PyErr {
    fn from(err: RegexError) -> PyErr {
        let msg = match err {
            RegexError::InvalidArg(s) => format!(
                "'{}' is not a valid argument of the form var=regex",
                s
            ),
            RegexError::InvalidRegex(s) => format!("Regex error {}", s),
        };
        PyValueError::new_err(msg)
    }
}

#[pyclass]
struct QueryTreePy {
    qt: QueryTree,
}

#[pyclass]
struct QueryResultPy {
    qr: QueryResult,
}

#[pyfunction(cpp = "false", regexes = "None")]
#[pyo3(text_signature = "(query, cpp, regexes)")]
fn parse_query(q: &str, cpp: bool, regexes: Option<Vec<&str>>) -> PyResult<QueryTreePy> {
    let vec_string = match regexes {
        Some(x) => x.into_iter().map(String::from).collect(),
        None => [].to_vec(),
    };

    let regex_constraints = process_regexes(&vec_string)?;
    let qt = parse_search_pattern(q, cpp, false, Some(regex_constraints))?;
    Ok(QueryTreePy { qt })
}

#[pyfunction]
#[pyo3(text_signature = "(p)")]
fn identifiers(p: &QueryTreePy) -> PyResult<Vec<String>> {
    Ok(p.qt.identifiers())
}

#[pyfunction(cpp = "false")]
#[pyo3(text_signature = "(p, source, cpp)")]
fn matches(p: &QueryTreePy, source: &str, cpp: bool) -> PyResult<Vec<QueryResultPy>> {
    let source_tree = crate::parse(source, cpp);

    let matches = p.qt.matches(source_tree.root_node(), source);

    let r = matches.into_iter().map(|qr| QueryResultPy { qr }).collect();

    Ok(r)
}

#[pyfunction]
#[pyo3(text_signature = "(q, key, source)")]
fn captured_value(p: &QueryResultPy, key: &str, source: &str) -> PyResult<Option<String>> {
    let r = match p.qr.value(key, source) {
        Some(x) => Some(x.to_string()),
        None => None,
    };
    Ok(r)
}

#[pyfunction(color = "None")]
#[pyo3(text_signature = "(q, source, color)")]
fn display(p: &QueryResultPy, source: &str, color: Option<bool>) -> PyResult<String> {
    if let Some(color_override) = color {
        colored::control::set_override(color_override);
    }
    let r = p.qr.display(source, 10, 10, false);
    colored::control::unset_override();
    Ok(r)
}

#[pymodule]
fn weggli(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<QueryTreePy>()?;
    m.add_function(wrap_pyfunction!(parse_query, m)?)?;
    m.add_function(wrap_pyfunction!(identifiers, m)?)?;
    m.add_function(wrap_pyfunction!(matches, m)?)?;
    m.add_function(wrap_pyfunction!(captured_value, m)?)?;
    m.add_function(wrap_pyfunction!(display, m)?)?;

    Ok(())
}
