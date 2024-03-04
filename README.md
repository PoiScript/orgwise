# `Orgwise`

Language server for org-mode, builtin with [`orgize`].

[`orgize`]: https://crates.io/crates/orgize

## Development

Requires `Rust 1.26+` for async trait.

### Server

```sh
$ cargo install --path .
```

### Client (vscode)

```sh
$ wasm-pack build -t web --no-default-features --features wasm
$ node build.mjs
$ pnpm run -C vscode package --no-dependencies
$ code --install-extension vscode/orgwise.vsix --force
```

## Supported features

1. Folding range

   - Fold headline, list, table, blocks

2. Document symbols

   - Headings

3. Formatting

4. Document link

   - File links

   - Source block `:tangle` arguments

   - Internal links

5. Code lens

   - Generate toc heading

   - Tangle/detanlge source block

   - Evaluate source block

6. Completion

   - Various blocks: `<a`, `<c`, `<C`, `<e`, `<E`, `<h`, `<l`, `<q`, `<s`, `<v`, `<I`

7. Commands

   - Show syntax tree

   - Preview org-mode file in html
