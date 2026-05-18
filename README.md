# Usage

## From Source:
```
cargo run -- "<expr>"
```

## Standalone
```
opteval "<expr>"
```

# Syntax

I wrote an extremely quick and dirty parser, so the syntax is kind of weird and the error messages aren't great. I'm sorry.

* Lambdas are notated with `!` and dots are simply dropped: `λx. x` -> `!x x`.
* Stacking multiple variables is done by writing multiple `!`-binders: `λab.a` -> `!a !b a`.
* Bindings are *not* just limited to one letter, so binding usages must have spaces in between: `λab.ab` -> `!a !b a b`.

# Examples

```
> cargo run -- "(!x x x) (!s !z s (s z))"
< (!x0 (!x1 (x0 (x0 (x0 (x0 x1))))))

> cargo run -- "(!x x x) (!a !b a)"
< (!x0 (!x1 (!x2 x1)))

> cargo run -- "(!f !x f (f x)) (!n !f !x f (n f x)) (!f !x f (f (f x)))"
< (!x0 (!x1 (x0 (x0 (x0 (x0 (x0 x1)))))))

> cargo run -- "!long_name long_name"
< (!x0 x0)
```

# If You Find a Counterterm (Incorrect Evaluation or Panic while Evaluating)

Please open an issue!
