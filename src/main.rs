mod congruence;
mod fol;
mod solver;
mod parser;

use std::io::Write;

use fol::*;
use solver::*;

fn main() {
    // let formula = r"f(f(f(a))) = a /\ f(f(f(f(f(a))))) = a /\ f(a) != a";
    // println!("{}: {}", parse_and_check_sat(formula), formula);

    let mut input = String::new();

    loop {
        print!(">>> ");
        input.clear();
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();

        let sort_a = Sort::new("A");
        let mut parser = parser::UnsortedParser::new(&sort_a);

        let input_trimmed = input.trim();

        match parser.parse_formula(input_trimmed) {
            Some((rest, formula)) => {
                if rest.trim().is_empty() {
                    println!("parsed: {}", formula);
                    println!("{}", QFEUFSolver::sat(&parser.get_language(), &formula));
                } else {
                    println!("failed to parse: {}", input_trimmed);
                }
            },
            None => {
                println!("failed to parse: {}", input_trimmed);
            },
        }
    }

    // let mut graph = CongruenceGraph::new();

    // // a = 0, b = 1, f = 2
    // graph.add_node(0, &vec![]);
    // graph.add_node(1, &vec![]);
    // graph.add_node(2, &vec![0]);
    // graph.add_node(2, &vec![1]);

    // graph.merge_congruence_classes(0, 1);
    
    // println!("{}", graph);

    // let mut graph = CongruenceGraph::new();

    // // a = 0, f = 1
    // graph.add_node(0, &vec![]);  // a
    // graph.add_node(1, &vec![0]); // f(a)
    // graph.add_node(1, &vec![1]); // f(f(a))
    // graph.add_node(1, &vec![2]); // f(f(f(a)))
    // graph.add_node(1, &vec![3]); // f(f(f(f(a))))
    // graph.add_node(1, &vec![4]); // f(f(f(f(f(a)))))

    // graph.merge_congruence_classes(0, 3);
    // graph.merge_congruence_classes(0, 5);
    
    // println!("{}", graph);

    // let sort_a = Sort::new("A");
    // let sort_b = Sort::new("B");
    // let relation_r = RelationSymbol::new("R", [&sort_a]);
    // let function_f = FunctionSymbol::new("f", [&sort_a], &sort_a);
    
    // let var_0 = Rc::new(Variable { index: 0, sort: sort_a.clone() });
    // let var_1 = Rc::new(Variable { index: 0, sort: sort_a.clone() });

    // let formula = Formula::UniversalQuantification(
    //     var_0.clone(),
    //     Rc::new(Formula::Conjunction(vec![
    //         Formula::new_relation_application(&relation_r, [&Term::new_variable(0, &sort_a)]),
    //         Formula::new_relation_application(&relation_r, [&Term::new_application(&function_f, [&Term::new_variable(0, &sort_b)])]),
    //     ])),
    // );

    // println!("{}", formula);
    // println!("{:#?}", formula.get_free_variables());

    // Example: f(f(f(a))) = a /\ f(f(f(f(f(a))))) = a /\ f(a) != a is unsat
    // let sort_a = Sort::new("A");
    // let constant_a = FunctionSymbol::new("a", &[], &sort_a);
    // let function_f = FunctionSymbol::new("f", &[&sort_a], &sort_a);
    
    // let language = Language::new(
    //     &[&sort_a],
    //     &[&constant_a, &function_f],
    //     &[],
    // );

    // let mut solver = QFEUFSolver::new(&language);

    // let a = Term::new_application(&constant_a, &[]);
    // let fa = Term::new_application(&function_f, &[&a]);
    // let ffa = Term::new_application(&function_f, &[&fa]);
    // let fffa = Term::new_application(&function_f, &[&ffa]);
    // let ffffa = Term::new_application(&function_f, &[&fffa]);
    // let fffffa = Term::new_application(&function_f, &[&ffffa]);

    // let t1 = solver.add_term(&a);
    // let t2 = solver.add_term(&fa);
    // let t3 = solver.add_term(&fffa);
    // let t4 = solver.add_term(&fffffa);
    // solver.add_equality(t3, t1);
    // solver.add_equality(t4, t1);
    
    // println!("{}", solver.congruence_graph);
    // println!("{}", solver.check_equality(t1, t2));

    // let mut parser = parser::UnsortedParser::new(&sort_a);

    // match parser.parse_term("f(a, f(a, b))") {
    //     Some(term) => println!("{}", term),
    //     None => print!("failed to parse"),
    // }

    // match parser.parse_formula(r"a = a /\ (b = b \/ c = c) /\ (d = d \/ e = e)") {
    //     Some(formula) => {
    //         println!("original: {}", formula);
    //         let dnf = QFEUFSolver::to_dnf(&formula);
    //         for clause in dnf {
    //             for (negated, formula) in clause {
    //                 if negated {
    //                     print!("¬({})  ", formula);
    //                 } else {
    //                     print!("{}  ", formula);
    //                 }
    //             }
    //             println!("")
    //         }
    //     },
    //     None => print!("failed to parse"),
    // }
}
