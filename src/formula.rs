// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-core/gchemol-core.note::*imports][imports:1]]
use crate::molecule::Molecule;

use std::collections::HashMap;
use std::iter::IntoIterator;
// imports:1 ends here

// core

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-core/gchemol-core.note::*core][core:1]]
fn get_reduced_symbols<'a, I>(symbols: I) -> HashMap<String, usize>
where
    I: IntoIterator,
    I::Item: std::fmt::Display,
{
    let symbols: Vec<_> = symbols
        .into_iter()
        .map(|item| format!("{:}", item))
        .collect();

    // 1. count symbols: CCCC ==> C 4
    let mut counts = HashMap::new();
    for x in symbols {
        let c = counts.entry(x).or_insert(0);
        *c += 1;
    }

    counts
}

fn get_reduced_formula<'a, I>(symbols: I) -> String
where
    I: IntoIterator,
    I::Item: std::fmt::Display,
{
    let counts = get_reduced_symbols(symbols);

    let mut syms: Vec<String> = Vec::new();
    let mut to_append = String::new();
    // 2. format the formula
    for (k, v) in counts {
        // 2.1 omit number if the count is 1: C1H4 ==> CH4
        let mut s = String::new();
        if v > 1 {
            s = v.to_string();
        }
        // 2.2 special treatments for C and H
        let reduced = format!("{}{}", k, s);
        if k == "C" {
            syms.insert(0, reduced);
        } else if k == "H" {
            to_append = reduced;
        } else {
            syms.push(reduced);
        }
    }
    // 3. final output
    syms.push(to_append);
    let formula = syms.join("");
    formula
}

#[test]
fn test_formula() {
    let symbols = vec!["C", "H", "C", "H", "H", "H"];
    let formula = get_reduced_formula(&symbols);
    assert_eq!("C2H4", formula);
    let symbols = vec!["C", "H", "C", "H", "H", "O", "H", "O"];
    let formula = get_reduced_formula(&symbols);
    assert_eq!("C2O2H4", formula);
}
// core:1 ends here

// api

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-core/gchemol-core.note::*api][api:1]]
impl Molecule {
    /// Return the molecule formula represented in string
    /// Return empty string if molecule containing no atom
    pub fn formula(&self) -> String {
        get_reduced_formula(self.symbols())
    }

    /// Return a hashmap for counting atom symbols.
    pub fn reduced_symbols(&self) -> HashMap<String, usize> {
        get_reduced_symbols(self.symbols())
    }
}
// api:1 ends here
