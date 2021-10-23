# Imperative Management of Declarative Packages #

Nix flakes are a really fantastic way to finely manage project dependencies.
However, writing Nix expressions to specify dependencies is subtle
and requires learning a whole new language. Can we do better?

Turns out *yes*.  Demo:


[![asciicast](https://asciinema.org/a/ZegsK3eFdDwz32mYorJEcOFoQ.svg)](https://asciinema.org/a/ZegsK3eFdDwz32mYorJEcOFoQ)



# Goal: Avoid writing boilerplate code #

The goal is to write an imperative CLI to automate the generation
of the declarative dependencies specified in a flake.

Every time I start a project, I find myself copying a
flake that works, and simply tweaking the
name and list of buildInputs. The goal is to streamline
that process and dodge writing any boilerplate code.

# How is this any different than `nix flake init` ? #

This is meant to be more powerful. Templates generate
boilerplate code, but without interaction. Interaction
is *key*. The end user should be able to get their
project up and running with minimal effort expended
towards figuring out how to get nix to play nicely with
their package and set of dependencies.

# How does this work? #

The idea is to have the user specify the type of dependencies,
flake inputs and outputs, and type of the package with `skim`,
then to output a flake. This flake is then validated with `rnix`.

If the user wants to modify an existing flake to add or remove
dependencies, this will also be possible. The flake shall be
parsed in with `rnix`, and the user will be able to modify it.

As of now, basically none of the features exist. I've only
got the proof of concept working: skim can be used for a cli
and rnix can be used to modify the AST.

Further down the line, I'd like to make this even more interactive.
This will involve querying github, crates.io, pypy, nixpkgs and more for packages,
then piping them into skim for selection based on language.

The hope is to also provide automatic support for pre-existing
nix expression generators such as node2nix, poetry2nix, cabal2nix,
and naersk.

# Dependencies #

I'm using the `rnix` parser to generate nix expressions,
and the `skim` fuzzy finder for the cli.

# Roadmap #

- [x] Proof of concept
- [ ] Flake Input management
  - [ ] Add inputs
  - [ ] Remove inputs
  - [ ] Change inputs
  - [ ] Query github
- [ ] BuildInput management
  - [ ] Query nixpkgs
  - [ ] Modify buildInputs
  - [ ] Add buildInputs
  - [ ] Delete buildInputs
- [ ] Specify flake outputs
- [ ] Support of specific languages:
    - [ ] Python
    - [x] Rust
    - [ ] Haskell

