/// Syntax of first-order logic

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hasher;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt;

#[derive(Debug)]
pub struct Sort {
    name: String,
}

pub struct RelationSymbol {
    name: String,
    input_sorts: Vec<Rc<Sort>>,
}

pub struct FunctionSymbol {
    name: String,
    input_sorts: Vec<Rc<Sort>>,
    output_sort: Rc<Sort>,
}

pub struct Language {
    sorts: Vec<Rc<Sort>>,
    function_symbols: Vec<Rc<FunctionSymbol>>,
    relation_symbols: Vec<Rc<RelationSymbol>>,
}

pub type VariableIndex = usize;

#[derive(Debug)]
pub struct Variable {
    pub index: VariableIndex,
    pub sort: Rc<Sort>,
}

pub enum Term {
    Variable(Rc<Variable>),
    Application(Rc<FunctionSymbol>, Vec<Rc<Term>>),
}

pub enum Formula {
    RelationApplication(Rc<RelationSymbol>, Vec<Rc<Term>>),
    Equality(Rc<Term>, Rc<Term>),
    Negation(Rc<Formula>),
    Implication(Rc<Formula>, Rc<Formula>),
    Equivalence(Rc<Formula>, Rc<Formula>),
    Conjunction(Vec<Rc<Formula>>), // 0-ary conjunction is true
    Disjunction(Vec<Rc<Formula>>), // 0-ary disjunction is false
    UniversalQuantification(Rc<Variable>, Rc<Formula>),
    ExistentialQuantification(Rc<Variable>, Rc<Formula>),
}

fn clone_vec_rc<T, const N: usize>(vec: [&Rc<T>; N]) -> Vec<Rc<T>> {
    vec.iter().map(|elem| (*elem).clone()).collect::<Vec<_>>()
}

fn merge_hash_maps<K: Eq + Hash, V>(map1: HashMap<K, V>, map2: HashMap<K, V>) -> HashMap<K, V> {
    map1.into_iter().chain(map2).collect()
}

impl Sort {
    pub fn new(name: &str) -> Rc<Sort> {
        Rc::new(Sort { name: name.to_string() })
    }
}

impl PartialEq for Sort {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Sort {}

impl Hash for Sort {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl RelationSymbol {
    pub fn new<const N: usize>(name: &str, input_sorts: [&Rc<Sort>; N]) -> Rc<RelationSymbol> {
        Rc::new(RelationSymbol {
            name: name.to_string(),
            input_sorts: clone_vec_rc(input_sorts),
        })
    }
}

impl FunctionSymbol {
    pub fn new<const N: usize>(name: &str, input_sorts: [&Rc<Sort>; N], output_sort: &Rc<Sort>) -> Rc<FunctionSymbol> {
        Rc::new(FunctionSymbol {
            name: name.to_string(),
            input_sorts: clone_vec_rc(input_sorts),
            output_sort: output_sort.clone(),
        })
    }
}

impl Language {
    pub fn new<const N: usize, const M: usize, const K: usize>(
        sorts: [&Rc<Sort>; N],
        function_symbols: [&Rc<FunctionSymbol>; M],
        relation_symbols: [&Rc<RelationSymbol>; K],
    ) -> Rc<Language> {
        Rc::new(Language {
            sorts: clone_vec_rc(sorts),
            function_symbols: clone_vec_rc(function_symbols),
            relation_symbols: clone_vec_rc(relation_symbols),
        })
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Variable) -> bool {
        self.index == other.index && self.sort == other.sort
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.sort.hash(state);
    }
}

pub type VariableSet = HashSet<Rc<Variable>>;

impl Term {
    pub fn new_variable(index: VariableIndex, sort: &Rc<Sort>) -> Rc<Term> {
        Rc::new(Term::Variable(Rc::new(Variable { index, sort: sort.clone() })))
    }

    pub fn new_application<const N: usize>(symbol: &Rc<FunctionSymbol>, arguments: [&Rc<Term>; N]) -> Rc<Term> {
        Rc::new(Term::Application(symbol.clone(), clone_vec_rc(arguments)))
    }

    pub fn collect_free_variables_in_set(&self, free_vars: &mut VariableSet) {
        match self {
            Term::Variable(variable) => {
                free_vars.insert(variable.clone());
            }
            Term::Application(_, arguments) => {
                for argument in arguments {
                    argument.collect_free_variables_in_set(free_vars);
                }
            }
        }
    }

    pub fn get_free_variables(&self) -> VariableSet {
        let mut free_vars = VariableSet::new();
        self.collect_free_variables_in_set(&mut free_vars);
        free_vars
    }
}

impl Formula {
    pub fn falsum() -> Rc<Formula> {
        Rc::new(Formula::Disjunction(Vec::new()))
    }

    pub fn verum() -> Rc<Formula> {
        Rc::new(Formula::Conjunction(Vec::new()))
    }

    pub fn new_relation_application<const N: usize>(symbol: &Rc<RelationSymbol>, arguments: [&Rc<Term>; N]) -> Rc<Formula> {
        Rc::new(Formula::RelationApplication(symbol.clone(), clone_vec_rc(arguments)))
    }

    pub fn collect_free_variables_in_set(&self, free_vars: &mut VariableSet) {
        match self {
            Formula::RelationApplication(_, arguments) => {
                for argument in arguments {
                    argument.collect_free_variables_in_set(free_vars);
                }
            },
            Formula::Equality(left, right) => {
                left.collect_free_variables_in_set(free_vars);
                right.collect_free_variables_in_set(free_vars);
            },
            Formula::Negation(formula) => formula.collect_free_variables_in_set(free_vars),
            Formula::Implication(left, right) => {
                left.collect_free_variables_in_set(free_vars);
                right.collect_free_variables_in_set(free_vars);
            },
            Formula::Equivalence(left, right) => {
                left.collect_free_variables_in_set(free_vars);
                right.collect_free_variables_in_set(free_vars);
            },
            Formula::Conjunction(conjuncts) => {
                for conjunct in conjuncts {
                    conjunct.collect_free_variables_in_set(free_vars);
                }
            },
            Formula::Disjunction(disjuncts) => {
                for disjunct in disjuncts {
                    disjunct.collect_free_variables_in_set(free_vars);
                }
            },
            Formula::UniversalQuantification(variable, body) => {
                let had_before = free_vars.contains(variable);
                body.collect_free_variables_in_set(free_vars);
                if !had_before {
                    free_vars.remove(variable);
                }
            },
            Formula::ExistentialQuantification(variable, body) => {
                let had_before = free_vars.contains(variable);
                body.collect_free_variables_in_set(free_vars);
                if !had_before {
                    free_vars.remove(variable);
                }
            },
        }
    }

    pub fn get_free_variables(&self) -> VariableSet {
        let mut free_vars = VariableSet::new();
        self.collect_free_variables_in_set(&mut free_vars);
        free_vars
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for RelationSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for input_sort in &self.input_sorts {
            write!(f, " {}", input_sort)?;
        }
        Ok(())
    }
}

impl fmt::Display for FunctionSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for input_sort in &self.input_sorts {
            write!(f, " {}", input_sort)?;
        }
        write!(f, " -> {}", self.output_sort)?;
        Ok(())
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{}:{}", self.index, self.sort)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Variable(variable) => write!(f, "{}", variable),
            Term::Application(symbol, arguments) => {
                write!(f, "{}(", symbol.name)?;
                for (i, argument) in arguments.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", argument)?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::RelationApplication(symbol, arguments) => {
                write!(f, "{}(", symbol.name)?;
                for (i, argument) in arguments.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", argument)?;
                }
                write!(f, ")")?;
                Ok(())
            },
            Formula::Equality(left, right) => write!(f, "{} = {}", left, right),
            Formula::Negation(formula) => write!(f, "¬({})", formula),
            Formula::Implication(left, right) => write!(f, "({} → {})", left, right),
            Formula::Equivalence(left, right) => write!(f, "({} ⇔ {})", left, right),
            Formula::Conjunction(conjuncts) => {
                if conjuncts.is_empty() {
                    write!(f, "⊤")
                } else if conjuncts.len() == 1 {
                    write!(f, "{}", conjuncts[0])
                } else {
                    write!(f, "(")?;
                    for (i, conjunct) in conjuncts.iter().enumerate() {
                        if i == 0 {
                            write!(f, "{}", conjunct)?;
                        } else {
                            write!(f, " ∧ {}", conjunct)?;
                        }
                    }
                    write!(f, ")")
                }
            }
            Formula::Disjunction(disjuncts) => {
                if disjuncts.is_empty() {
                    write!(f, "⊥")
                } else if disjuncts.len() == 1 {
                    write!(f, "{}", disjuncts[0])
                } else {
                    write!(f, "(")?;
                    for (i, disjunct) in disjuncts.iter().enumerate() {
                        if i == 0 {
                            write!(f, "{}", disjunct)?;
                        } else {
                            write!(f, " ∨ {}", disjunct)?;
                        }
                    }
                    write!(f, ")")
                }
            },
            Formula::UniversalQuantification(variable, body) =>
                write!(f, "∀{} ({})", variable, body),
            Formula::ExistentialQuantification(variable, body) =>
                write!(f, "∃{} ({})", variable, body),
        }
    }
}

// TODO
// 1. format
// 2. free variables
// 3. (potentially not capture free) substitution
// 4. sort check
// 5. Skolemization
