use std::rc::Rc;
use std::iter;
use std::collections::HashMap;

use crate::fol::*;

use nom::*;
use nom::sequence::delimited;
use nom::character::complete::multispace0;
use nom::bytes::complete::tag;

macro_rules! ws {
    ($x:expr) => {
        delimited::<_, _, _, _, error::Error<&str>, _, _, _>(multispace0, $x, multispace0)
    }
}

macro_rules! sep_list {
    ($self:expr, $x:ident, $y:expr, $input:expr) => {
        match $self.$x($input) {
            IResult::Err(_) => IResult::Ok(($input, vec![])),
            IResult::Ok((input, x)) => {
                let mut input = input;
                let mut xs = vec![x];
                loop {
                    match $y(input) {
                        IResult::Err(_) => return IResult::Ok((input, xs)),
                        IResult::Ok((rest, _)) => {
                            let (rest, x) = $self.$x(rest)?;
                            input = rest;
                            xs.push(x);
                        }
                    }
                }
            }
        }
    };
}

pub struct UnsortedParser {
    sort: Rc<Sort>,
    arity_map: HashMap<String, Rc<FunctionSymbol>>,
}

impl UnsortedParser {
    pub fn new(sort: &Rc<Sort>) -> UnsortedParser {
        UnsortedParser {
            sort: sort.clone(),
            arity_map: HashMap::new(),
        }
    }

    fn identifier<'a>(&self, input: &'a str) -> IResult<&'a str, &'a str> {
        character::complete::alphanumeric1(input)
    }
    
    fn arguments<'a>(&mut self, input: &'a str) -> IResult<&'a str, Vec<Rc<Term>>> {
        let (input, _) = ws!(tag("("))(input)?;
        let (input, arguments) = self.terms(input)?;
        let (input, _) = ws!(tag(")"))(input)?;
        IResult::Ok((input, arguments))
    }
    
    /// Parses a term, a term is either an application "f(<term>, ...)"
    /// or a constant "a"
    fn term<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Term>> {
        let (input, symbol) = self.identifier(input)?;
        let (input, arguments) = self.arguments(input).or(IResult::Ok((input, vec![])))?;
    
        // create a new function symbol
        let function_symbol = if self.arity_map.contains_key(symbol) {
            // TODO: better error handling
            assert!(self.arity_map[symbol].arity() == arguments.len(), "function symbol {} is used with different arities", symbol);
            self.arity_map[symbol].clone()
        } else {
            let new_symbol = FunctionSymbol::new(
                symbol,
                &iter::repeat(&self.sort).take(arguments.len()).collect::<Vec<&Rc<Sort>>>(), 
                &self.sort,
            );
            self.arity_map.insert(symbol.to_string(), new_symbol.clone());
            new_symbol.clone()
        };
    
        IResult::Ok((input, Term::new_application(&function_symbol, &arguments.iter().collect::<Vec<_>>())))
    }
    
    fn terms<'a>(&mut self, input: &'a str) -> IResult<&'a str, Vec<Rc<Term>>> {
        sep_list!(self, term, ws!(tag(",")), input)
    }
    
    fn equality<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, left) = self.term(input)?;
        let (input, _) = ws!(tag("="))(input)?;
        let (input, right) = self.term(input)?;
        IResult::Ok((input, Formula::new_equality(&left, &right)))
    }

    fn neg_equality<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, left) = self.term(input)?;
        let (input, _) = ws!(tag("!="))(input)?;
        let (input, right) = self.term(input)?;
        IResult::Ok((input, Formula::new_negation(&Formula::new_equality(&left, &right))))
    }

    fn paren_formula<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, _) = ws!(tag("("))(input)?;
        let (input, formula) = self.formula(input)?;
        let (input, _) = ws!(tag(")"))(input)?;
        IResult::Ok((input, formula))
    }

    fn atomic_formula<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        self.equality(input).or(self.neg_equality(input)).or(self.paren_formula(input))
    }

    fn negation<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, _) = ws!(tag("!"))(input)?;
        let (input, formula) = self.atomic_formula(input)?;
        IResult::Ok((input, Formula::new_negation(&formula)))
    }

    fn unary<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        self.negation(input).or(self.atomic_formula(input))
    }

    fn conjunction_list<'a>(&mut self, input: &'a str) -> IResult<&'a str, Vec<Rc<Formula>>> {
        sep_list!(self, unary, ws!(tag("/\\")), input)
    }
    
    fn conjunction<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, conjuncts) = self.conjunction_list(input)?;
        IResult::Ok((input, Formula::new_conjunction(&conjuncts.iter().collect::<Vec<_>>())))
    }

    fn disjunction_list<'a>(&mut self, input: &'a str) -> IResult<&'a str, Vec<Rc<Formula>>> {
        sep_list!(self, conjunction, ws!(tag("\\/")), input)
    }

    fn disjunction<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, disjuncts) = self.disjunction_list(input)?;
        IResult::Ok((input, Formula::new_disjunction(&disjuncts.iter().collect::<Vec<_>>())))
    }

    fn implication_or_disjunction<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        let (input, left) = self.disjunction(input)?;
        match ws!(tag("->"))(input) {
            IResult::Err(_) => IResult::Ok((input, left)),
            IResult::Ok((input, _)) => {
                let (input, right) = self.disjunction(input)?;
                IResult::Ok((input, Formula::new_implication(&left, &right)))
            }
        }
    }

    fn formula<'a>(&mut self, input: &'a str) -> IResult<&'a str, Rc<Formula>> {
        self.implication_or_disjunction(input)
    }

    pub fn parse_term(&mut self, input: &str) -> Option<Rc<Term>> {
        match self.term(input) {
            IResult::Ok((_, term)) => Some(term.clone()),
            _ => None,
        }
    }

    /// Parse a quantifier free formula with terms looking like f(a, f(a, b))
    pub fn parse_formula(&mut self, input: &str) -> Option<Rc<Formula>> {
        match self.formula(input) {
            IResult::Ok((_, formula)) => Some(formula.clone()),
            _ => None,
        }
    }

    /// Return the language containing all function symbols currently constructed
    pub fn get_language(&self) -> Rc<Language> {
        Language::new(&[&self.sort], &self.arity_map.values().collect::<Vec<_>>(), &[])
    }
}
