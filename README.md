# Motivation? #

Nix flakes are a really fantastic way to finely manage project dependencies.
However, writing Nix expressions to specify dependencies is subtle
and requires learning a whole new language. Can we do better?

Turns out *yes*. Demo:


[![asciicast](https://asciinema.org/a/GkoYll3a9mT7R6264xymdxuP2.svg)](https://asciinema.org/a/GkoYll3a9mT7R6264xymdxuP2)



# GOAL: AVOID WRITING BOILERPLATE CODE #

The goal is to write an imperative CLI to automate the generation
of the declarative dependencies specified in a flake.

Every time I start a project, I find myself copying a
flake that works, and simply tweaking the 
name and list of buildInputs. The goal is to streamline
that process and dodge writing any boilerplate code.

# How does this work? #

The idea is to specify the type of dependencies, flake inputs,
flake outputs, and type of the package with FZF, then to output
a flake.

This will involve querying github and nixpkgs for packages,
then piping them into FZF for selection based on language.

The hope is to also provide automatic support for pre-existing
nix expression generators such as node2nix, poetry2nix, cabal2nix,
and naersk.

# Dependencies #

I'm using the `hnix` parser to generate nix expressions,
and `turtle` to query user input with `fzf`.

# Roadmap #

- [x] Proof of concept
- [ ] Add inputs
- [ ] Add to buildInputs
- [ ] Specify flake outputs
- [ ] Support of specific languages TODO
