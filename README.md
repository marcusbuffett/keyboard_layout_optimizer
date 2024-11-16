# Keyboard Layout Optimizer

Forked from [dariogoetz/keyboard_layout_optimizer](https://github.com/dariogoetz/keyboard_layout_optimizer) to add support for the [sval](https://github.com/sval-keyboard/sval) keyboard.

## Evaluating a layout

To evaluate a layout, you run a command like this:

```sh
cargo watch -x 'run  --bin evaluate -- --layout-config config/keyboard/sval.yml --exclude-chars " 0123456789=(){}[]" "q0a1z w2sbx e3dtc r4fgv uhj5m iyk67 onl89 p={}(["'
```

It's a bit annoying, but the way to fill in extra spaces which don't use an alpha key, is to use these chars: `0123456789=(){}[]`, you can see above I've used them in the qwerty layout string.

## Optimizing a new layout

To optimize a new layout, you run a command like this:

```sh
cargo watch -x 'run  --bin optimize_genetic -- --layout-config config/keyboard/sval.yml --exclude-chars " 0123456789=(){}[]" --start-layout "q0a1z w2sbx e3dtc r4fgv uhj5m iyk67 onl89 p={}(["'
```

You need an initial layout, although in practice it doesn't really matter, unless the initial layout is very close to optimal. The example above begins with the standard qwerty layout.

You can see a list of potentially interesting options for optimization with `cargo run  --bin optimize_genetic -- --help`. For example, to optimize a new layout while keeping certain characters fixed, for example vim users who want to keep "hjkl" where they are while optimizing everything else, you can use the `--fix` argument.

```sh
cargo watch -x 'run  --bin optimize_genetic -- --layout-config config/keyboard/sval.yml --exclude-chars " 0123456789=(){}[]" --fix 'hjkl' --start-layout "q0a1z w2sbx e3dtc r4fgv uhj5m iyk67 onl89 p={}(["'
```


## Modifying for personal use

There are a number of places where I've made assumptions that may not be true for everyone.

The first place is in the [key costs](./config/keyboard/sval.yml) in the Svalboard keyboard file. Check them out, you may have totally different values. I generally love center and south, don't love inward as much, then north, then last of all is outward (except pinky and index).

There's also the [finger repeats](./layout_evaluation/src/metrics/bigram_metrics/finger_repeats.rs) metric. It's got a bunch of different ways to measure the cost of a single-finger bigram, with assumptions like:
1) Center-south rolls are nice
2) Center-inward rolls are nice-ish
3) Most other rolls are bad
4) Inward-outward or outward-inward rolls are very bad
5) etc

There's also the [scissoring](./layout_evaluation/src/metrics/bigram_metrics/scissoring.rs) metric. Basically it just tests if you're going north on one finger then south on the other. You may care more or less about this than I do.

Finally there's the [finger movement](./layout_evaluation/src/metrics/bigram_metrics/movement_pattern.rs) metric. This measures the difficulty of switching between fingers on the same hand. The specific values can be tweaked in the [default optimization values file](./config/optimization/default.yml).





