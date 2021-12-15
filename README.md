Nelson-Oppen Algorithm for Congruence Closure and a Solver for QF_EUF
---

## Build and Run

To run the tool, make sure [`Cargo`](https://doc.rust-lang.org/cargo/) is intalled.

Use the following command to compile and run the tool:
```
$ cargo run
```

If it works successfully, you should see a prompt for formulas `>>> `.

The current interface only support unsorted signatures.
For instance,
```
>>> !(a = b /\ b = c -> f(a) = f(c))
parsed: ¬(((a() = b() ∧ b() = c()) → f(a()) = f(c())))
unsat
```
The tool will parse the formula and collect all function/constant symbols.
If all uses of the symbols have consistent arities, then the tool can infer an unsorted signature.
In the above example, the signature will be `{ a, b, c, f }` where `a, b, c` are nullary and `f` is unary.

More examples:
```
>>> f(f(f(a))) = a /\ f(f(f(f(f(a))))) = a /\ f(a) != f(f(a))
parsed: (f(f(f(a()))) = a() ∧ f(f(f(f(f(a()))))) = a() ∧ ¬(f(a()) = f(f(a()))))
unsat
```

```
>>> f(a, b) = c /\ g(c) = d /\ g(f(a, b)) != d
parsed: (f(a(), b()) = c() ∧ g(c()) = d() ∧ ¬(g(f(a(), b())) = d()))
unsat
```

```
>>> f(a) = f(f(f(a))) /\ f(a) != f(f(a))
parsed: (f(a()) = f(f(f(a()))) ∧ ¬(f(a()) = f(f(a()))))
sat
```

## Components

The tool has the following components
- `congruence.rs` contains a naive implementation of the Nelson-Oppen algorithm.
- `fol.rs` contains definitions of the AST for a many-sorted first-order logic.
- `parser.rs` contains a parser for formulas.
- `solver.rs` contains the main solver loop, which basically reduces an input formulas to DNF and checks the unsatifiability of each disjunct.
- `main.rs` contains the entrypoint of the tool.

The congruence closure algorithm I have implemented is similar to Nelson-Oppen but more straightforward and inefficient.
Whenever an equality is added, I am enumerating all nodes in the E-DAG to find congruent pairs.
