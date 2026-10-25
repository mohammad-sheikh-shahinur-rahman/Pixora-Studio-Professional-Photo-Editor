# Contributing to Pixora Studio

Thank you for your interest in improving **Pixora Studio** — a native, professional
photo editor written entirely in Rust. Contributions of all sizes are welcome:
features, bug fixes, tests, documentation, translations, and triage.

## Getting started

1. Read [`AGENTS.md`](AGENTS.md) — the engineering constitution (golden rules,
   layering, testing gates) — then the docs it points to.
2. See [`docs/contributing.md`](docs/contributing.md) for the detailed contributor
   guide (clean-room rules, the "add a command" checklist, commit style).
3. Set up a build:

   ```sh
   cargo build
   cargo test -p photocraft-engine          # quick smoke of the command layer
   ```

   Full requirements (Windows/macOS/Linux, fonts, corpora): `docs/development.md`.

## Ground rules (short version)

- **Never crash.** Non-test code never `unwrap()`s, `panic!`s or indexes
  input-derived data unchecked; malformed files and bad parameters must return
  errors the user can act on.
- **Everything is a command.** User-visible behaviour lives in the engine's
  command registry, so the UI, CLI, control channel and automation all share it.
- **No format or colour assumptions.** Bit depth and colour space are runtime
  data; colour goes through the ICC layer, never hard-coded sRGB paths.
- **Clean-room.** Behaviour and look are studied from public specs and
  observation; never copy competitor code or assets.
- **Tests are the gate.** Every change ships with tests, and the parity floors
  must not regress.
- **Keep it native.** No JavaScript/TypeScript, no Electron/Tauri/webview shells.

## Submitting changes

- Work on a branch; keep commits small and descriptive
  (`feat:`, `fix:`, `docs:`, `perf:` …).
- Run the check commands from `AGENTS.md §5` for the crates you touched before
  opening a pull request; CI runs the full set, including real-file format
  corpora.
- For behaviour-affecting changes, include before/after evidence (test output,
  screenshots for UI changes).

## Reporting bugs and security issues

- Bugs: open a [GitHub Issue](https://github.com/mohammad-sheikh-shahinur-rahman/Pixora-Studio-Professional-Photo-Editor/issues)
  with the version, OS, steps to reproduce and, for file problems, the affected
  document.
- Security: follow [`SECURITY.md`](SECURITY.md) — please do not open public
  issues for undisclosed vulnerabilities.

## Licensing

By contributing you agree your work is dual-licensed under
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE), matching the rest of the
project. Third-party material must be permissively licensed and documented in
[`ATTRIBUTION.md`](ATTRIBUTION.md).
