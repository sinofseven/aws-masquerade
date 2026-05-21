# clapの書き方を現代化

## 目的
我流の書き方になっているので、流行に合わせたい

## 要望
clapの書き方を今の流行に合わせて欲しい

## プラン

### 方針

clap 4.6.1 を builder API + 自前の `base::Cmd` トレイトで使い回している現状を、derive API ベース (`#[derive(Parser)]` / `#[derive(Subcommand)]` / `#[derive(Args)]` / `#[derive(ValueEnum)]`) に全面置換する。挙動 (サブコマンド名・positional 引数・フラグ・help テキスト・`CredentialOutputTarget` の possible values と aliases) は完全に保つ。

- `Cli` / `Commands` を `main.rs` に置き、`Cli::parse().command.run()` で dispatch。
- 4 群 (`configure` / `source` / `target` / `assume`) は Args 構造体 + Subcommand enum。
- `arg_required_else_help = true` は各 Args に明示。
- `CredentialOutputTarget` は derive ValueEnum 化。`PowerShell`/`SharedCredentials` は `#[value(name = ...)]` で kebab-case 自動命名を抑止。aliases も完全保持。
- `base::Cmd` トレイトと `variables::cmd::*` 定数モジュールは削除。
- `path.rs` の migration 警告は文字列リテラル化。
- 既存の 1 ファイル 1 コマンド構造 (`cmd/source/list.rs` 等) は保持。
- `cargo` feature は害がないので残す。
- `base::Validation`、`cmd/target/list.rs` の v0/v1 分岐、`CredentialOutputTarget::new()` (dead code) は範囲外として維持。

### 変更ファイル

- `Cargo.toml` (derive feature 追加)
- `src/main.rs` (Cli/Commands 導入)
- `src/base/cmd.rs` (削除)、`src/base/mod.rs` (cmd 参照削除)
- `src/cmd/mod.rs`、`src/cmd/configure/mod.rs`、`src/cmd/source/{mod,list,show}.rs`、`src/cmd/target/{mod,list,show}.rs`、`src/cmd/assume.rs`
- `src/variables.rs` (cmd モジュール削除)
- `src/path.rs` (cmd_configure 参照を文字列化)
- `src/models/configuration/v1.rs` (impl ValueEnum を derive 化)
- `CLAUDE.md` (アーキテクチャ説明を更新)

### 検証 (テストが無いのでスモークテスト)

`cargo build` / `cargo clippy` の clean、各サブコマンドの `--help` 表示確認、`configure path` 等の動作確認、`-c` の possible values / aliases の維持確認、`arg_required_else_help` の維持確認。

詳細な調査結果・会話内容・実装ログは `log.md` を参照。

## 完了サマリー

完了日時: 2026-05-18T22:33:07+09:00

clap 4.6.1 を builder API + 自前 `base::Cmd` トレイト + `variables::cmd::*` 文字列定数で運用していた構造を、derive API ベース (`#[derive(Parser)]` / `Subcommand` / `Args` / `ValueEnum`) に全面移行。

主な変更:
- `Cli` / `Commands` を `main.rs` に導入し `Cli::parse().command.run()` で dispatch。
- 各群 (`configure` / `source` / `target` / `assume`) を `XxxArgs` 構造体 + `XxxCommand` enum に再構成。
- `base::Cmd` トレイト (`src/base/cmd.rs`) と `variables::cmd::*` モジュールを削除。
- `CredentialOutputTarget` を手書き `impl ValueEnum` から `#[derive(ValueEnum)]` に置換。`PowerShell` / `SharedCredentials` の possible value 名と全 aliases を完全保持。
- `arg_required_else_help = true` を各 Args 構造体に明示し、引数なし呼び出しで help が出る挙動を維持。
- CLAUDE.md の「サブコマンド構造とディスパッチ規約」「既知の注意点」を新パターンに合わせて更新。

検証: `cargo build` / `cargo clippy --all-targets` ともクリーン (警告ゼロ)。全サブコマンドの `--help` 出力・positional 引数 (`<TARGET_NAME>`, `<SOURCE_NAME>`)・`-c/--credential-output` の possible values と aliases (`j`/`Json` 等)・`arg_required_else_help` の挙動・`--version` がすべて現行と一致することを動作確認済み。

範囲外として維持: `base::Validation` トレイト、`cmd/target/list.rs` の v0/v1 明示分岐、`CredentialOutputTarget::new(&str)` (dead code)。
