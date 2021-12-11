mod congruence;
mod fol;

use std::rc::Rc;

use congruence::CongruenceGraph;
use fol::*;

fn main() {
    // let mut graph = CongruenceGraph::new();

    // // a = 0, b = 1, f = 2
    // graph.add_parent(0, &vec![]);
    // graph.add_parent(1, &vec![]);
    // graph.add_parent(2, &vec![0]);
    // graph.add_parent(2, &vec![1]);

    // graph.merge_congruence_classes(0, 1);
    
    // println!("{}", graph);

    let mut graph = CongruenceGraph::new();

    // a = 0, f = 1
    graph.add_parent(0, &vec![]);  // a
    graph.add_parent(1, &vec![0]); // f(a)
    graph.add_parent(1, &vec![1]); // f(f(a))
    graph.add_parent(1, &vec![2]); // f(f(f(a)))
    graph.add_parent(1, &vec![3]); // f(f(f(f(a))))
    graph.add_parent(1, &vec![4]); // f(f(f(f(f(a)))))

    graph.merge_congruence_classes(0, 3);
    graph.merge_congruence_classes(0, 5);
    
    println!("{}", graph);

    let sort_a = Sort::new("A");
    let sort_b = Sort::new("B");
    let relation_r = RelationSymbol::new("R", [&sort_a]);
    let function_f = FunctionSymbol::new("f", [&sort_a], &sort_a);
    
    let var_0 = Rc::new(Variable { index: 0, sort: sort_a.clone() });
    let var_1 = Rc::new(Variable { index: 0, sort: sort_a.clone() });

    let formula = Formula::UniversalQuantification(
        var_0.clone(),
        Rc::new(Formula::Conjunction(vec![
            Formula::new_relation_application(&relation_r, [&Term::new_variable(0, &sort_a)]),
            Formula::new_relation_application(&relation_r, [&Term::new_application(&function_f, [&Term::new_variable(0, &sort_b)])]),
        ])),
    );

    println!("{}", formula);
    println!("{:#?}", formula.get_free_variables());
}
