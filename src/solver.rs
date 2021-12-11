use std::borrow::Borrow;
use std::panic;
use std::rc::Rc;
use std::fmt;

use crate::congruence::*;
use crate::fol::*;

type Literal = (bool, Rc<Formula>);
type Clause = Vec<Literal>;
type ClauseList = Vec<Clause>;

pub enum SatResult {
    Sat,
    Unsat,
    Unknown,
}

impl fmt::Display for SatResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SatResult::Sat => write!(f, "sat"),
            SatResult::Unsat => write!(f, "unsat"),
            SatResult::Unknown => write!(f, "unknown"),
        }
    }
}

/// A solver for quantifier-free theory of equality and uninterpreted functions
pub struct QFEUFSolver {
    pub congruence_graph: CongruenceGraph,
    symbol_table: Vec<Rc<FunctionSymbol>>,
}

impl QFEUFSolver {
    pub fn new(language: &Rc<Language>) -> QFEUFSolver {
        // TODO: handle relation
        QFEUFSolver {
            congruence_graph: CongruenceGraph::new(),
            symbol_table: language.iter_function_symbols().map(|x| x.clone()).collect(),
        }
    }

    /// Add a symbol if it does not exist
    pub fn add_symbol(&mut self, symbol: &Rc<FunctionSymbol>) -> SymbolIndex {
        if let Some(index) = self.symbol_table.iter().position(|x| x == symbol) {
            return index;
        }
        self.symbol_table.push(symbol.clone());
        return self.symbol_table.len() - 1;
    }

    pub fn get_symbol_id(&self, symbol: &Rc<FunctionSymbol>) -> SymbolIndex {
        for (i, other) in self.symbol_table.iter().enumerate() {
            if other == symbol {
                return i;
            }
        }
        panic!("symbol {} not found", symbol);
    }

    /// Add a term to the congruence graph and return the node index
    pub fn add_term(&mut self, term: &Rc<Term>) -> NodeIndex {
        match term.borrow() {
            Term::Variable(_) => panic!("variable not supported"),
            Term::Application(symbol, arguments) => {
                let symbol_id = self.get_symbol_id(symbol);
                if arguments.is_empty() {
                    // constant
                    return self.congruence_graph.add_node(symbol_id, &vec![]);
                } else {
                    // application
                    let mut children = vec![];
                    for argument in arguments {
                        children.push(self.add_term(argument));
                    }
                    return self.congruence_graph.add_node(symbol_id, &children);
                }
            }
        }
    }

    pub fn check_equality(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        return self.congruence_graph.get_congruent_class(node1) == self.congruence_graph.get_congruent_class(node2);
    }

    pub fn add_equality(&mut self, node1: NodeIndex, node2: NodeIndex) {
        self.congruence_graph.merge_congruence_classes(node1, node2);
    }

    pub fn flip_literals(clauses: &mut ClauseList) {
        for clause in clauses {
            for (negation, _) in clause.iter_mut() {
                *negation = !*negation;
            }
        }
    }

    fn clone_clause(clause: &Clause) -> Clause {
        clause.iter().map(|(negated, formula)| (*negated, formula.clone())).collect::<Vec<_>>()
    }

    /// Convert a formula to an equivalent CNF formula
    pub fn to_cnf(formula: &Rc<Formula>) -> ClauseList {
        match formula.borrow() {
            Formula::Negation(formula) => {
                let mut cnf = QFEUFSolver::to_dnf(formula);
                QFEUFSolver::flip_literals(&mut cnf);
                cnf
            },

            Formula::Implication(left, right) =>
                QFEUFSolver::to_cnf(&Formula::new_disjunction(
                    &[&Formula::new_negation(left), right],
                )),

            Formula::Equivalence(left, right) =>
                QFEUFSolver::to_cnf(&Formula::new_conjunction(
                    &[
                        &Formula::new_implication(left, right),
                        &Formula::new_implication(right, left),
                    ],
                )),
                
            Formula::Conjunction(conjuncts) =>
                conjuncts.iter().map(|conjunct| QFEUFSolver::to_cnf(conjunct)).collect::<Vec<_>>().concat(),

            Formula::Disjunction(disjuncts) => {
                if disjuncts.is_empty() {
                    return vec![vec![]];
                }

                let first_cnf = QFEUFSolver::to_cnf(&disjuncts[0]);
                let rest_cnf = QFEUFSolver::to_cnf(&Rc::new(Formula::Disjunction(disjuncts[1..].to_vec())));
                let mut cnf = vec![];

                for clause1 in &first_cnf {
                    for clause2 in &rest_cnf {
                        cnf.push([QFEUFSolver::clone_clause(clause1), QFEUFSolver::clone_clause(clause2)].concat());
                    }
                }
                
                cnf
            },

            // will not do deeper if hit atomic formula or quantifiers
            _ => vec![vec![(false, formula.clone())]],
        }
    }

    /// Convert a formula to an equivalent DNF formula
    pub fn to_dnf(formula: &Rc<Formula>) -> ClauseList {
        match formula.borrow() {
            Formula::Negation(formula) => {
                let mut dnf = QFEUFSolver::to_cnf(formula);
                QFEUFSolver::flip_literals(&mut dnf);
                dnf
            },

            Formula::Implication(left, right) =>
                QFEUFSolver::to_dnf(&Formula::new_disjunction(
                    &[&Formula::new_negation(left), right],
                )),

            Formula::Equivalence(left, right) =>
                QFEUFSolver::to_dnf(&Formula::new_conjunction(
                    &[
                        &Formula::new_implication(left, right),
                        &Formula::new_implication(right, left),
                    ],
                )),
                
            Formula::Conjunction(conjuncts) => {
                if conjuncts.is_empty() {
                    return vec![vec![]];
                }

                let first_dnf = QFEUFSolver::to_dnf(&conjuncts[0]);
                let rest_dnf = QFEUFSolver::to_dnf(&Rc::new(Formula::Conjunction(conjuncts[1..].to_vec())));
                let mut dnf = vec![];

                for clause1 in &first_dnf {
                    for clause2 in &rest_dnf {
                        dnf.push([QFEUFSolver::clone_clause(clause1), QFEUFSolver::clone_clause(clause2)].concat());
                    }
                }
                
                dnf
            },

            Formula::Disjunction(disjuncts) =>
                disjuncts.iter().map(|disjunct| QFEUFSolver::to_dnf(disjunct)).collect::<Vec<_>>().concat(),

            // will not do deeper if hit atomic formula or quantifiers
            _ => vec![vec![(false, formula.clone())]],
        }
    }

    /// Check if a clause (conjunction) is satisfiable
    pub fn clause_sat(language: &Rc<Language>, clause: &Clause) -> SatResult {
        let mut solver = QFEUFSolver::new(language);
        let mut equalities = vec![];
        let mut negated_equalities = vec![];

        // add all terms
        for (negated, formula) in clause {
            if let Formula::Equality(left, right) = formula.borrow() {
                let node1 = solver.add_term(left);
                let node2 = solver.add_term(right);
                if *negated {
                    negated_equalities.push((node1, node2));
                } else {
                    equalities.push((node1, node2));
                }
            } else {
                panic!("Relation not supported");
            }
        }

        for (node1, node2) in equalities {
            solver.add_equality(node1, node2);
        }

        for (node1, node2) in negated_equalities {
            if solver.check_equality(node1, node2) {
                return SatResult::Unsat;
            }
        }

        SatResult::Sat
    }

    /// Check if the given QF_EUF formula is satisfiable
    pub fn sat(language: &Rc<Language>, formula: &Rc<Formula>) -> SatResult {
        // TODO: instead of DNF, use a faster way to search for sat assignments
        let dnf = QFEUFSolver::to_dnf(formula);

        for clause in dnf {
            if let SatResult::Sat = QFEUFSolver::clause_sat(language, &clause) {
                return SatResult::Sat;
            }
        }

        SatResult::Unsat
    }
}
