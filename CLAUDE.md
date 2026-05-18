# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

`aws-masquerade` は AWS AssumeRole 用の Rust 製 CLI ツール。現在 `v1-master` ブランチで v1 への全面改修中で、`master` への直接マージ前にここで反復している。v1 では設定ファイルを JSON (v0) から TOML (v1) に変更し、`account` を `source` (AssumeRole を実行する側) と `target` (AssumeRole の対象) に分割した。

README.md の TODO リストが v1 完成までの作業項目。未着手のもの: `source`/`target` の `add`/`edit`/`remove` サブコマンド (備考にあるとおり、これらを設けるか自体も検討中)。

## ビルド/実行

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo run -- <subcommand>    # 開発中の実行 (例: cargo run -- configure path)
cargo check                  # 型チェックのみ
cargo clippy                 # lint
```

テストコードは無いため `cargo test` は無実装。動作確認は実際のサブコマンドを `cargo run` で叩く。

サードパーティライセンスファイルは `cargo about generate about.hbs` で生成 (CI でのみ実行)。

## アーキテクチャ

### サブコマンド構造とディスパッチ規約

`main.rs` がトップレベルの 4 サブコマンド (`configure`, `source`, `target`, `assume`) を登録し、各構造体が `base::Cmd` トレイトを実装してディスパッチする一貫したパターンを採る:

```rust
pub trait Cmd {
    const NAME: &'static str;          // variables::cmd::* の定数を参照すること
    fn subcommand() -> Command;        // clap の Command を返す
    fn run(args: &ArgMatches) -> Result<(), String>;
}
```

新しいサブコマンドを追加するときは:
1. `variables.rs` の `cmd::<group>::sub_command` にコマンド名定数を追加 (文字列をハードコードしない)
2. `src/cmd/<group>/<name>.rs` に `Cmd` を実装した構造体を作成
3. `src/cmd/<group>/mod.rs` の `subcommand()`/`run()` に登録

ネストしたグループ (`configure`, `source`, `target`) は `Cmd` を実装した構造体を更に内部で持ち、同じトレイト経由でディスパッチしている (`src/cmd/configure/mod.rs` が参考例)。

### 設定ファイルのバージョン管理とマイグレーション

`~/.config/aws-masquerade/config.toml` (v1) を優先し、無ければ `~/.config/aws-masquerade/config.json` (v0) を読む。`src/path.rs::get_current_path_masquerade_config()` がこの判定を行い、v0 を返すときは stderr に migration を促す警告を出す。

設定読み込みは原則 `models::configuration::load_configuration()` を経由する。これが v0 を読み込んだ場合は自動で v1 に in-memory 変換するので、呼び出し側は常に `v1::Configuration` を扱える。例外として `cmd/target/list.rs` だけは明示的に両バージョンを分岐している (歴史的経緯と思われる) ので、新規コードでは `load_configuration()` を使うこと。

`v0::MasqueradeConfig::migrate()` の挙動: v0 の `account` ひとつを v1 の `source` + `target` のペアに分解し、`source_profile` (無ければアカウント名) を `source.name` として使う。`HashSet` を経由するので同じ profile を共有する複数の account はひとつの `source` に集約される。

### Validation

`base::Validation` トレイトを `Configuration`/`Source`/`Target` が実装。`Configuration::validate()` は `source.name`/`target.name` の一意性と `target.source` が `source.name` のいずれかを指していることをチェックする。設定を使う前にこれを必ず呼ぶ (`assume`/`configure validate`/`source list`/`target list` 等で実施済み)。

### AssumeRole 実行フロー (`cmd/assume.rs`)

1. `TARGET_NAME` で `target` を引き、`target.source` から `source` を引く。
2. `generate_sdk_config(source)` が `aws_config::SdkConfig` を組む:
   - `aws_access_key_id` + `aws_secret_access_key` が両方あれば `Credentials::new(..., "Static")` を使う
   - `profile` が指定されていれば `ProfileFileCredentialsProvider` で上書き (両方指定された場合は profile が勝つ実装)
   - region は `source.region` → 無ければ `us-east-1`
3. `sts.assume_role()` を呼ぶ。`source.mfa_arn` がある場合、`mfa_secret` から TOTP を `crate::totp::generate` で生成、無ければ stdin で MFA トークンを対話入力。
4. `target.credential_output` (CLI `-c` で上書き可) に応じて結果を出力:
   - `Json`/`Bash`/`Fish`/`PowerShell`: stdout に出力 (それぞれ `eval $(...)`、`... | source`、`... | Invoke-Expression` でシェルに食わせる前提のフォーマット)
   - `SharedCredentials`: `~/.aws/credentials` (`AWS_SHARED_CREDENTIALS_FILE` で上書き可) に `serde_ini` で直接書き込み、`target.name` をプロファイル名にする

`tokio::runtime::Runtime::new().block_on(...)` でその場限りの runtime を作って async 実行している (`#[tokio::main]` は使わない)。

### TOTP (`src/totp.rs`)

`core.save_totp_counter_history = true` の場合、`~/.config/aws-masquerade/.totp_count_history.json` に `secret -> counter` を記録し、同じカウンタが既に使用済みなら次の 30 秒境界まで `std::thread::sleep` で待つ。AWS は同一 TOTP コードの再利用を弾くため、この機構が必要。

### エラー型

全体で `Result<_, String>` を使用 (独自エラー型なし)。`map_err(|e| format!(...))` で文脈を付与する慣習なので、新規コードでも揃えること。

## CI / リリース

`.github/workflows/build.yml`:
- `master` への push、`v*` タグ、PR、手動実行で起動。
- ビルドターゲット: linux x86_64 (musl), linux arm64 (musl), linux arm (musl, Docker 経由), macOS aarch64, windows x86_64。
- タグ push の場合のみアーティファクトをアップロード → `packaging` job で `THIRD_PARTY_LICENSES.html` + `README.md` + `README_ja.md` + `LICENSE` を同梱して zip 化し、GitHub Release (draft) を作成。
- `README_ja.md` を参照しているが現状リポジトリには無いため、リリース時のみ作る運用か未追加。新規ファイルを同梱したい場合はこの job の `cp ...` 行も更新する。

## 開発ワークフロー

開発作業は `kanban-kit` プラグインで管理する。詳細手順は `/kanban-kit:kanban` スキルの `references/kanban-workflow.md` を参照。

### kanban ディレクトリ構造

```
kanban/
  {xxxx}_{title}/
    {xxxx}_{title}.md   # タスクファイル（ユーザーが作成）
    log.md              # 作業ログ（Claude が記録）
```

- `xxxx`: 4桁0パディング連番（例: `0001`）
- タスクファイルには `## 目的`（Why）と `## 要望`（How/What）を記載する

### 主なスキル

- `/kanban-kit:add-kanban` — 新規タスクファイルを作成する
- `/kanban-kit:kanban` — 未完了タスクを実行する（未指定時は番号最大のものを選択）

未完了タスクの判定: `## 完了サマリー` セクションを含まないタスクファイル。

## 既知の注意点

- `variables.rs` の `cmd::target::sub_command::REMOTE` は綴り誤り (`REMOVE` のはず) だが現状未使用なので未修正。新しく `target remove` を実装するならここを直すこと。
- `Configuration::validate` のエラーメッセージに `tource`/`dupplicate` 等のタイポあり。修正する際はテストが無いので grep で参照箇所を確認すること。
