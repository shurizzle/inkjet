<h1 align="center">Inkjet</h1>
<h3 align="center">A batteries-included syntax highlighting library for Rust, based on <code>tree-sitter</code>.</h3>
<p align="center">
<img src=".github/logo.png" width="256">
</p>

<p align="center">
<img src="https://img.shields.io/crates/v/inkjet">
<img src="https://img.shields.io/github/actions/workflow/status/SomewhereOutInSpace/inkjet/rust.yml">
<img src="https://img.shields.io/crates/l/inkjet">
</p>

## Features

- Language grammars are linked into the executable as C functions - no need to load anything at runtime!
- Pluggable formatters. Inkjet includes a formatter for HTML, and writing your own is easy.
- Highlight into a new `String` or a `std::io::Write`/`std::fmt::Write`, depending on your use case.
- Specify languages explicitly (from an `enum`) or look them up using a token like `"rs"` or `"rust"`.
- ~~Extremely cursed `build.rs`~~

## Included Languages

Inkjet comes bundled with support for over seventy languages, and it's easy to add more - see the FAQ section.

<details>
    <summary><strong style="cursor: pointer">Click to expand...</strong></summary>

| Name | Recognized Tokens |
| ---- | ------- |
| Ada  | `ada`   |
| Assembly (generic) | `asm` |
| Astro | `astro` |
| Awk | `awk` |
| Bash | `bash` |
| BibTeX | `bibtex`, `bib` |
| Bicep | `bicep` |
| Blueprint | `blueprint`, `blp` |
| C | `c`, `h` |
| Cap'N Proto | `capnp` |
| Clojure | `clojure`, `clj`, `cljc` |
| C# | `c_sharp`, `c#`, `csharp`, `cs` |
| Common Lisp | `commonlisp`, `common-list`, `cl`, `lisp` |
| C++ | `c++`, `cpp`, `hpp`, `h++`, `cc`, `hh` |
| CSS | `css` |
| Cue | `cue` |
| D | `d`, `dlang` |
| Dart | `dart` |
| Diff | `diff` |
| Dockerfile | `dockerfile`, `docker` |
| EEx | `eex` |
| Emacs Lisp | `elisp`, `emacs-lisp`, `el` |
| Elixir | `ex`, `exs`, `leex` |
| Elm | `elm` |
| Erlang | `erl`, `hrl`, `es`, `escript` |
| Forth | `forth`, `fth` |
| Fortran | `fortran`, `for` |
| GDScript | `gdscript`, `gd` |
| Gleam | `gleam` |
| GLSL | `glsl` |
| Go | `go`, `golang` |
| Haskell | `haskell`, `hs` |
| HCL | `hcl`, `terraform` |
| HEEx | `heex` |
| HTML | `html`, `htm` |
| IEx | `iex` |
| INI | `ini` |
| JavaScript | `javascript`, `js` |
| JSON | `json` |
| Kotlin | `kotlin`, `kt`, `kts` |
| LaTeX | `latex`, `tex` |
| LLVM | `llvm` |
| Lua | `lua` |
| GNU Make | `make`, `makefile`, `mk` |
| MatLab | `matlab`, `m` |
| Meson | `meson` |
| Nim | `nim` |
| Nix | `nix` |
| OCaml | `ocaml`, `ml` |
| OCaml Interface | `ocaml_interface`, `mli` |
| OpenSCAD | `openscad`, `scad` |
| PHP | `php` |
| ProtoBuf | `protobuf`, `proto` |
| Python | `python`, `py` |
| R | `r` |
| Racket | `racket`, `rkt` |
| Regex | `regex` |
| Ruby | `ruby`, `rb` |
| Rust | `rust`, `rs` |
| Scala | `scala` |
| Scheme | `scheme`, `scm`, `ss` |
| SCSS | `scss` |
| SQL (Generic) | `sql` |
| Swift | `swift` |
| TOML | `toml` |
| TypeScript | `typescript`, `ts` |
| WAST (WebAssembly Script) | `wast` |
| WAT (WebAssembly Text) | `wat`, `wasm` |
| x86 Assembly | `x86asm`, `x86` |
| WGSL | `wgsl` |
| YAML | `yaml` |
| Zig | `zig` |

</details>

In addition to these languages, Inkjet also offers the [`Runtime`](https://docs.rs/inkjet/latest/inkjet/enum.Language.html#variant.Runtime) and [`Plaintext`](https://docs.rs/inkjet/latest/inkjet/enum.Language.html#variant.Plaintext) languages.
- `Runtime` wraps a `fn() -> &'static HighlightConfiguration` pointer, which is used to resolve the language at (you guessed it) runtime.
- `Plaintext` enables cheap no-op highlighting. It loads the `diff` grammar under the hood, but provides no highlighting queries. It's aliased to `none` and `nolang`.

## Cargo Features
- (Default) `html` - enables the bundled HTML formatter, which depends on `v_htmlescape` and the `theme` feature.
- (Default) `theme` - enables the theme API, which depends on the `html` feature, `ahash`, `toml` and `serde`.
- (Default) `all-languages` - enables all languages.
- `language-{name}` - enables the specified language.
    - If you want to only enable a subset of the included languages, you'll have to set `default-features=false` and manually re-add each language you want to use.
## FAQ

### *"Why is Inkjet so large?"*

Parser sources generated by `tree-sitter` can grow quite big, with some being dozens of megabytes in size. Inkjet has to bundle these sources for all the languages it supports, so it adds up. (According to `loc`, there are over 23 *million* lines of C code!)

If you need to minimize your binary size, consider disabling languages that you don't need. Link-time optimization can also shave off a few megabytes.

### *"Why is Inkjet taking so long to build?"*

Because it has to compile and link in dozens of C/C++ programs (the parsers and scanners for every language Inkjet bundles.)

However, after the first build, these artifacts will be cached and subsequent builds should be much faster.

### *"Why does highlighting require a mutable reference to the highlighter?*

Under the hood, Inkjet creates a `tree-sitter` highlighter/parser object, which in turn dynamically allocates a chunk of working memory. Using the same highlighter for multiple simultaneous jobs would therefore cause all sorts of nasty UB.

If you want to highlight in parallel, you'll have to create a clone of the highlighter for each thread.

### *"A language I want to highlight isn't bundled with Inkjet!"*

Assuming that you or someone else has implemented a highlighting-ready `tree-sitter` grammar for the language you want, adding it to Inkjet is easy! Just open an issue asking for it to be added, linking to the grammar repository for the language.

Alternatively, you can use [`Language::Runtime`](https://docs.rs/inkjet/latest/inkjet/enum.Language.html#variant.Runtime), which will allow you to use grammars not bundled with Inkjet.

Other notes:
- Inkjet currently only supports grammar repositories that check in the parser generated by `tree-sitter` (in order to avoid a build-time dependency on `node`/`npm`.)
- Inkjet requires that the grammar include (at minimum) a `highlights.scm` query targeted at the base `tree-sitter` library. Extended queries (such as those from `nvim-treesitter`) will not work.
- I will not support blockchain/smart contract languages like Solidity. Please take your scam enablers elsewhere.

## Building
For normal use, Inkjet will compile automatically just like any other crate.

However, if you have forked the repository and want to update the bundled languages, you'll need to set some environment variables:
- `INKJET_REDOWNLOAD_LANGS` will wipe the `languages/` directory and redownload everything from scratch.
  - Currently, this only works on *nix. You will need `git`, `sed` and `wget` installed. (Git clones the grammar repositories, while `sed` and `wget` are used in miniature setup scripts for some languages.)
- `INKJET_REBUILD_LANGS_MODULE` will wipe `src/languages.rs` and regenerate it from scratch.
- `INKJET_REBUILD_FEATURES` will generate a file called `features` in the crate root, containing all the individual language features (ready to be pasted into `Cargo.toml`.)

The value of these variables doesn't matter - they just have to be set. 

Additionally:
- You will need to pass the `--all-features` flag to `cargo` for these to work - by default, the development parts of the build script are not compiled.
- I recommend running `cargo build -vv` when redownloading languages, so the script's progress is visible.

## Acknowledgements
- Inkjet would not be possible without `tree-sitter` and the ecosystem of grammars surrounding it.
- Many languages are only supported thanks to the highlighting queries created by the [Helix](https://github.com/helix-editor/helix) project.
