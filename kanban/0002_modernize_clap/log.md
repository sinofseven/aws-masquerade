# 0002_modernize_clap 作業ログ

開始時刻: 2026-05-18T22:25:55+09:00
完了時刻: 2026-05-18T22:33:07+09:00

## タスク概要

kanban ファイル (`0002_modernize_clap.md`) の要望:
- 目的: 我流の書き方になっているので、流行に合わせたい
- 要望: clapの書き方を今の流行に合わせて欲しい

## 調査結果

### Cargo.toml (clap の依存)

```
clap = {version = "4.6.1", features=["cargo"]}
```

`cargo` feature のみ有効で `derive` feature は未指定。`derive` を追加すれば `#[derive(Parser)]` 系が使えるようになる。

### `src/base/cmd.rs` - 独自 `Cmd` トレイト

```rust
use clap::{ArgMatches, Command};

pub trait Cmd {
    const NAME: &'static str;
    fn subcommand() -> Command;
    fn run(args: &ArgMatches) -> Result<(), String>;
}
```

これは derive API が標準で提供する Parser/Subcommand/Args の機能を手で再発明したパターン。`NAME` 定数によりサブコマンド名を識別、`subcommand()` で clap の Command を組み立て、`run(&ArgMatches)` で実行。

### `src/base/mod.rs` / `src/base/model.rs`

```rust
// mod.rs
mod cmd;
mod model;
pub use cmd::Cmd;
pub use model::Validation;
```

```rust
// model.rs
pub trait Validation {
    fn validate(&self) -> Result<(), String>;
}
```

`Validation` トレイトは clap と無関係 (model 用の validation インターフェース)。残す。

### `src/main.rs`

```rust
fn main() -> Result<(), String> {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Configure::subcommand())
        .subcommand(Source::subcommand())
        .subcommand(Target::subcommand())
        .subcommand(Assume::subcommand())
        .get_matches();
    match matches.subcommand() {
        Some((Configure::NAME, args)) => Configure::run(args),
        Some((Source::NAME, args)) => Source::run(args),
        Some((Target::NAME, args)) => Target::run(args),
        Some((Assume::NAME, args)) => Assume::run(args),
        _ => unreachable!(""),
    }
}
```

`command!()` マクロで Cargo.toml の metadata を取り込む。トップレベルに `subcommand_required(true)` と `arg_required_else_help(true)` がついている。

### `src/variables.rs`

```rust
pub mod cmd {
    pub mod configure {
        pub const NAME: &str = "configure";
        pub mod sub_command {
            pub const PATH: &str = "path";
            pub const VALIDATE: &str = "validate";
            pub const MIGRATE: &str = "migrate";
        }
    }
    pub mod assume { pub const NAME: &str = "assume"; }
    pub mod source {
        pub const NAME: &str = "source";
        pub mod sub_command {
            pub const LIST: &str = "list";
            pub const SHOW: &str = "show";
            pub const ADD: &str = "add";
            pub const EDIT: &str = "edit";
            pub const REMOVE: &str = "remove";
        }
    }
    pub mod target { /* 同上 (REMOTE という綴り誤りあり) */ }
}

pub mod models { ... }
pub mod output { ... }
```

`cmd::*` 配下のすべての NAME 定数は clap derive で不要 (enum variant 名から自動的に kebab-case のサブコマンド名が生成される)。`models` と `output` は変更不要。

`cmd::source::sub_command::ADD/EDIT/REMOVE` および `cmd::target::sub_command::ADD/EDIT/REMOTE(誤字)` は現状参照箇所が一切ないため、`cmd` モジュール削除と同時に消える。CLAUDE.md の「既知の注意点」で言及されている `REMOTE` 綴り誤りも自然に解消。

### `src/path.rs` (`variables::cmd::*` 参照)

```rust
use crate::variables::cmd::configure as cmd_configure;
// ...
eprintln!(
    "{}",
    format!(
        "$ aws-masquerade {} {}",
        cmd_configure::NAME,
        cmd_configure::sub_command::MIGRATE
    ).yellow()
);
```

migration 警告メッセージのみ参照。文字列リテラル `"configure"` `"migrate"` に置換するだけで済む。

### `src/cmd/mod.rs`

```rust
mod assume;
mod configure;
mod source;
mod target;

pub use assume::Assume;
pub use configure::Configure;
pub use source::Source;
pub use target::Target;
```

各 cmd モジュールから単体構造体 (Cmd トレイト実装) を re-export。derive 化後は `pub use` で Args 型 (or Commands enum) を re-export する形に変わる。

### `src/cmd/configure/mod.rs`

3 つの leaf (Path/Validate/Migrate) すべてがこの 1 ファイルに `struct Path; struct Validate; struct Migrate;` として定義され、各々 `impl Cmd` している。`subcommand_required(true).arg_required_else_help(true)` も親で設定。

- `Path::run` → `path::get_current_path_masquerade_config()` を呼んで PathBuf を println。
- `Validate::run` → `load_configuration()` + `configure.validate()`。
- `Migrate::run` → v0 ファイル読み込み → `MasqueradeConfig::migrate()` → v1 を save。

`Validate` と `Migrate` の `subcommand()` は `.about(...)` が無い (現状の help がそうなっている)。`Path` だけ about あり。derive 化でも同じ表記にする。

### `src/cmd/source/mod.rs` / `list.rs` / `show.rs`

`mod.rs`:
```rust
pub struct Source;
impl Cmd for Source {
    const NAME: &'static str = source::NAME;
    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("Commands related to source configuration")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(List::subcommand())
            .subcommand(Show::subcommand())
    }
    fn run(args: &ArgMatches) -> Result<(), String> {
        match args.subcommand() {
            Some((List::NAME, sub_args)) => List::run(sub_args),
            Some((Show::NAME, sub_args)) => Show::run(sub_args),
            _ => unreachable!("..."),
        }
    }
}
```

`list.rs` (`List::run`): config validate → `source.iter().map(|s| &s.name)` を JSON pretty で出力。

`show.rs` (`Show::run`): `arg!(<SOURCE_NAME>)` を読み、config から source を find、JSON pretty で出力。

### `src/cmd/target/mod.rs` / `list.rs` / `show.rs`

source と同パターン。ただし `cmd/target/list.rs` だけ `load_configuration()` を使わず明示的に v0/v1 分岐:

```rust
let (path, version) = crate::path::get_current_path_masquerade_config()?;
let text = crate::fs::load_text(&path)?;
let config = match version {
    Version::V0 => ConfigV0::new(&text)?.migrate(),
    Version::V1 => ConfigV1::new(&text)?,
};
```

CLAUDE.md に「歴史的経緯と思われる」「新規コードでは load_configuration() を使うこと」と明記。derive 化しても `run` 本体ロジックは触らない。

### `src/cmd/assume.rs`

最大のサブコマンド (約 400 行)。clap 引数定義部分:

```rust
fn subcommand() -> Command {
    Command::new(Self::NAME)
        .about("execute assume role")
        .arg(arg!(<TARGET_NAME>))
        .arg(
            arg!(-c <CREDENTIAL_OUTPUT> "output of assume role result")
                .long("credential-output")
                .value_parser(clap::builder::EnumValueParser::<CredentialOutputTarget>::new()),
        )
}
```

`run` の本体は AWS SDK 呼び出し + 出力フォーマット (Bash/Fish/PowerShell/SharedCredentials/Json) で巨大だが、引数取り出しの部分:

```rust
let name_target: &String = args.get_one("TARGET_NAME").unwrap();
let credential_output = match args.get_one::<CredentialOutputTarget>("CREDENTIAL_OUTPUT") {
    Some(output) => output,
    None => &target.credential_output,
};
```

derive 化で `AssumeArgs { target_name: String, credential_output: Option<CredentialOutputTarget> }` にする。`unwrap()` も `Option` 経由処理もすべて derive 由来の型で自然に表現できる。

### `src/models/configuration/v1.rs` の `CredentialOutputTarget`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CredentialOutputTarget {
    #[serde(rename = "json")] Json,
    #[serde(rename = "bash")] Bash,
    #[serde(rename = "fish")] Fish,
    PowerShell,
    SharedCredentials,
}

impl CredentialOutputTarget {
    pub fn new(text: &str) -> Result<CredentialOutputTarget, String> { /* case-insensitive parse */ }
}

impl clap::ValueEnum for CredentialOutputTarget {
    fn value_variants<'a>() -> &'a [Self] { /* 5 variants */ }
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        // json (alias: j, Json), bash (alias: b, Bash), fish (alias: f, Fish),
        // PowerShell (alias: p), SharedCredentials (alias: s)
    }
}
```

possible_value の名前と aliases を derive で完全再現する必要がある。clap derive の rename はデフォルト kebab-case なので、`Json`/`Bash`/`Fish` (単語1つ) はそのまま小文字、`PowerShell` は `power-shell` に、`SharedCredentials` は `shared-credentials` に化ける → `#[value(name = "PowerShell")]` 等で上書きが必要。

`CredentialOutputTarget::new(&str)` メソッドは現在 grep で参照箇所が見つからず dead code。範囲外なので削除しない (kanban の要望は clap modernize、dead code 除去は別)。

`Display` 実装 (`CliOutputTarget` 側) は clap と無関係なので維持。

### 既存 help 出力 (参考)

```
$ aws-masquerade --help
AWS Assume Role CLI Tool

Usage: aws-masquerade <COMMAND>

Commands:
  configure  Commands related to configuration files
  source     Commands related to source configuration
  target     commands related to target configuration
  assume     execute assume role
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ aws-masquerade assume --help
execute assume role

Usage: aws-masquerade assume [OPTIONS] <TARGET_NAME>

Arguments:
  <TARGET_NAME>

Options:
  -c, --credential-output <CREDENTIAL_OUTPUT>
          output of assume role result [possible values: json, bash, fish, PowerShell, SharedCredentials]
  -h, --help
          Print help
```

derive 化後もこれと一致することを検証する。

## 実装プラン

### 全体方針

clap 4.6.1 を builder API + 自前 `base::Cmd` トレイト + `variables::cmd::*` の文字列定数で運用している構造を、derive API ベースに全面置換する。挙動 (CLI として観測できるすべて: サブコマンド名、positional 引数、フラグ、help テキスト、possible values、aliases、`arg_required_else_help`) は完全保持。

### 検討した代替案

**A. 段階的移行 (一部だけ derive 化)**
- 例えば assume だけ derive 化、他は builder API のまま。
- 却下理由: 「我流の書き方を是正」という目的に対して中途半端。`Cmd` トレイトが残るとパターンが二重化する。

**B. derive 化と同時にファイル構造も再編 (1 ファイル / 1 群)**
- `cmd/source/{mod,list,show}.rs` → `cmd/source.rs` 1 本に集約。
- 利点: derive API の旨味 (短いコマンドを inline で書ける) を活かせる。
- 却下理由: 差分が大きくなり、kanban の要望「clap の書き方を現代化」を超える。git history も乱れる。ユーザーが希望すれば次の kanban で扱う。

**C. 採用案: ファイル構造は維持し、clap の書き方だけ全面 derive 化**
- 各 leaf ファイル (`list.rs`/`show.rs` 等) は `pub struct XxxArgs` (引数なしならユニット構造体 or ユニット variant) と `pub fn run` をエクスポート。
- 親 `mod.rs` に Subcommand enum を置き dispatch。
- `Cli::parse().command.run()` で全体ディスパッチ。

### 具体的な構造案

**`main.rs`:**
```rust
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Commands related to configuration files
    Configure(ConfigureArgs),
    /// Commands related to source configuration
    Source(SourceArgs),
    /// commands related to target configuration
    Target(TargetArgs),
    /// execute assume role
    Assume(AssumeArgs),
}

impl Commands {
    fn run(self) -> Result<(), String> {
        match self {
            Commands::Configure(a) => a.run(),
            Commands::Source(a) => a.run(),
            Commands::Target(a) => a.run(),
            Commands::Assume(a) => a.run(),
        }
    }
}

fn main() -> Result<(), String> {
    Cli::parse().command.run()
}
```

**`cmd/configure/mod.rs`:**
```rust
#[derive(clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ConfigureArgs {
    #[command(subcommand)]
    command: ConfigureCommand,
}

#[derive(clap::Subcommand)]
enum ConfigureCommand {
    /// show config file path
    Path,
    Validate,
    Migrate,
}

impl ConfigureArgs {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            ConfigureCommand::Path => run_path(),
            ConfigureCommand::Validate => run_validate(),
            ConfigureCommand::Migrate => run_migrate(),
        }
    }
}

fn run_path() -> Result<(), String> { /* 現行 Path::run の中身 */ }
fn run_validate() -> Result<(), String> { /* 現行 */ }
fn run_migrate() -> Result<(), String> { /* 現行 */ }
```

**`cmd/source/mod.rs`:**
```rust
mod list;
mod show;

#[derive(clap::Args)]
#[command(arg_required_else_help = true, about = "Commands related to source configuration")]
pub struct SourceArgs {
    #[command(subcommand)]
    command: SourceCommand,
}

#[derive(clap::Subcommand)]
enum SourceCommand {
    /// list source name
    List,
    /// show detail of a source
    Show(show::ShowArgs),
}

impl SourceArgs {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            SourceCommand::List => list::run(),
            SourceCommand::Show(a) => show::run(a),
        }
    }
}
```

`list.rs` / `show.rs` は単純な `pub fn run` (および show は `pub struct ShowArgs`)。

**`cmd/target/mod.rs`:** source と同パターン。`target/list.rs` の v0/v1 分岐は維持。

**`cmd/assume.rs`:**
```rust
#[derive(clap::Args)]
pub struct AssumeArgs {
    target_name: String,
    #[arg(short = 'c', long = "credential-output", help = "output of assume role result")]
    credential_output: Option<CredentialOutputTarget>,
}

impl AssumeArgs {
    pub fn run(self) -> Result<(), String> { /* 現行 run の中身 */ }
}
```

**`models/configuration/v1.rs` の CredentialOutputTarget:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, clap::ValueEnum)]
pub enum CredentialOutputTarget {
    #[serde(rename = "json")]
    #[value(name = "json", alias = "j", alias = "Json")]
    Json,
    #[serde(rename = "bash")]
    #[value(name = "bash", alias = "b", alias = "Bash")]
    Bash,
    #[serde(rename = "fish")]
    #[value(name = "fish", alias = "f", alias = "Fish")]
    Fish,
    #[value(name = "PowerShell", alias = "p")]
    PowerShell,
    #[value(name = "SharedCredentials", alias = "s")]
    SharedCredentials,
}
```

`impl clap::ValueEnum for CredentialOutputTarget` ブロックは削除。

### 削除対象

- `src/base/cmd.rs` (ファイルごと)
- `src/base/mod.rs` の `mod cmd; pub use cmd::Cmd;`
- `src/variables.rs` の `pub mod cmd { ... }` ブロック
- `src/models/configuration/v1.rs` の `impl clap::ValueEnum for CredentialOutputTarget` ブロック
- 各 cmd ファイルの `use crate::base::Cmd;` / `use crate::variables::cmd::*` の import

### 書き換え対象

- `Cargo.toml`: `features=["cargo"]` → `features=["cargo", "derive"]`
- `src/path.rs`: `cmd_configure::NAME` `cmd_configure::sub_command::MIGRATE` 参照を文字列リテラル化、`use crate::variables::cmd::configure as cmd_configure;` 行を削除。
- `CLAUDE.md`: 「サブコマンド構造とディスパッチ規約」を新パターンに合わせて書き直し。「既知の注意点」の `variables.rs` の `REMOTE` 綴り誤り言及は不要に (cmd モジュール自体が消える)。

## プランニング経緯

初回提案 (advisor に相談する前の素案):
- derive API 全面移行
- `Cmd` トレイトと `variables::cmd::*` 削除
- ValueEnum を derive 化
- ファイル構造維持

advisor からのフィードバック (リジェクトではなく補強):
1. `arg_required_else_help = true` は derive で別途明示が必要 (`subcommand_required` は暗黙、こちらは違う)
2. ValueEnum の kebab-case 自動命名に注意 (`PowerShell`/`SharedCredentials` は `#[value(name = ...)]` で上書き)
3. `cargo` feature の去就を明示
4. スモークテスト計画を plan に含める
5. ファイル構造維持/再編の判断を plan に明示
6. `Validation`/`target/list.rs` 分岐/`CredentialOutputTarget::new` 維持を明示

最終プラン (`misty-snuggling-widget.md` および kanban `## プラン` セクション) はこれらすべてを反映。ユーザーはこの最終プランを承認。

## 会話内容

1. ユーザーが `/kanban-kit:kanban 0002` 起動。
2. Claude が `0002_modernize_clap/0002_modernize_clap.md` を読み込み、`## 完了サマリー` がないことと `## 目的` セクションがあることを確認。
3. Claude が EnterPlanMode に入る前にコードベース調査:
   - `Cargo.toml`、`main.rs`、`src/base/{cmd,model,mod}.rs`、`src/variables.rs`、`src/cmd/**`、`src/models/configuration/v1.rs`、`src/path.rs` を読む
   - `CredentialOutputTarget::new` の参照を grep → 参照なし (dead code)
   - 現行 help 出力を `cargo run -- --help` / `cargo run -- assume --help` で確認
4. Claude が advisor に方針を相談、上記 6 点のフィードバックを受領。
5. Claude が EnterPlanMode に入り、plan ファイル (`/Users/yuta/.claude/plans/misty-snuggling-widget.md`) を書いて ExitPlanMode で承認を得る。
6. ユーザーが plan を承認。実装フェーズへ移行。
7. Claude が kanban ファイルへ `## プラン` セクションを追記、log.md を作成。

## 編集したファイル

タスク順:

1. `kanban/0002_modernize_clap/0002_modernize_clap.md` — `## プラン` セクション追記
2. `kanban/0002_modernize_clap/log.md` — 本ファイル作成
3. `Cargo.toml` — `clap` の features に `derive` を追加 (`features=["cargo", "derive"]`)
4. `src/models/configuration/v1.rs` — `impl clap::ValueEnum for CredentialOutputTarget` ブロックを削除し、`#[derive(...)]` に `clap::ValueEnum` を追加。各 variant に `#[value(name = ..., alias = ..., alias = ...)]` を付与。`PowerShell` / `SharedCredentials` には `name = "PowerShell"` / `name = "SharedCredentials"` を明示して kebab-case 自動命名を抑止。
5. `src/cmd/assume.rs` — `pub struct Assume` + `impl Cmd` を `pub struct AssumeArgs { target_name: String, credential_output: Option<CredentialOutputTarget> }` に置換。`run` を `impl AssumeArgs` のメソッドに。`use crate::base::Cmd` / `use crate::variables::cmd::assume` / `use clap::{arg, ArgMatches, Command}` を削除。`generate_sdk_config` / `exec_assume` / `exec_output` / `JsonCredential` は変更なし。
6. `src/cmd/configure/mod.rs` — `pub struct Configure/Path/Validate/Migrate` + `impl Cmd` を `pub struct ConfigureArgs { #[command(subcommand)] command: ConfigureCommand }` と `enum ConfigureCommand { Path, Validate, Migrate }` に置換。各 leaf の処理を `fn run_path/run_validate/run_migrate` に分離。`#[command(arg_required_else_help = true)]` を `ConfigureArgs` に付与。
7. `src/cmd/source/list.rs` — `pub struct List` + `impl Cmd` を `pub fn run()` に簡素化。
8. `src/cmd/source/show.rs` — `pub struct ShowArgs { source_name: String }` + `pub fn run(args: ShowArgs)`。
9. `src/cmd/source/mod.rs` — `pub struct SourceArgs { #[command(subcommand)] command: SourceCommand }` + `enum SourceCommand { List, Show(show::ShowArgs) }` + `impl SourceArgs::run`。
10. `src/cmd/target/list.rs` — `pub fn run()` に簡素化。v0/v1 分岐ロジックは維持。
11. `src/cmd/target/show.rs` — `pub struct ShowArgs { target_name: String }` + `pub fn run`。
12. `src/cmd/target/mod.rs` — `pub struct TargetArgs` + `enum TargetCommand` の同パターン。
13. `src/cmd/mod.rs` — `pub use` を `AssumeArgs / ConfigureArgs / SourceArgs / TargetArgs` に変更。
14. `src/main.rs` — `command!()` ベースの builder を `#[derive(Parser)] struct Cli` + `#[derive(Subcommand)] enum Commands` に置換。`Cli::parse().command.run()` で dispatch。`use base::Cmd` 削除、`use clap::command` 削除、`use clap::Parser` 追加。
15. `src/base/cmd.rs` — ファイル削除 (`rm` で)。
16. `src/base/mod.rs` — `mod cmd; pub use cmd::Cmd;` を削除、`mod model; pub use model::Validation;` のみ残す。
17. `src/variables.rs` — `pub mod cmd { ... }` ブロック全体を削除。`models` と `output` モジュールは維持。
18. `src/path.rs` — `use crate::variables::cmd::configure as cmd_configure` を削除。migration 警告の `format!("$ aws-masquerade {} {}", cmd_configure::NAME, cmd_configure::sub_command::MIGRATE)` を `"$ aws-masquerade configure migrate"` の文字列リテラルに置換。
19. `CLAUDE.md` — 「サブコマンド構造とディスパッチ規約」を derive API 版に書き換え。「既知の注意点」から `variables.rs::cmd::target::sub_command::REMOTE` の言及を削除 (cmd モジュール自体が消えたので)。代わりに `CredentialOutputTarget::new` が dead code になった点と derive ValueEnum の name 上書きに関する注意を追加。

## 実行したコマンド

- `find /Users/yuta/space/private/aws-masquerade/src -type f -name "*.rs"` — ソース構造の把握
- `grep -rn "CredentialOutputTarget::new"` — dead code 確認
- `grep -rn "use crate::variables"` / `grep -rn "base::Cmd"` — リファクタリング対象の参照確認
- `cargo run --quiet -- --help` / `cargo run --quiet -- assume --help` — 現行 help 出力の取得 (調査時)
- `cargo check` — 各段階でのコンパイル確認 (複数回)
- `cargo build` — 最終ビルド確認 (clean: 4.17s で完走)
- `cargo clippy --all-targets` — lint 確認 (clean、警告なし)
- 動作確認:
  - `cargo run --quiet -- --help` — トップレベルヘルプ
  - `cargo run --quiet -- configure --help` / `configure path --help` 相当
  - `cargo run --quiet -- source --help` / `source show --help`
  - `cargo run --quiet -- target --help` / `target show --help`
  - `cargo run --quiet -- assume --help`
  - `cargo run --quiet -- --version` — `aws-masquerade 0.3.1` を確認
  - `cargo run --quiet -- configure path` — 設定パスが現行どおり出力されること確認
  - `cargo run --quiet -- configure` (引数なし) — `arg_required_else_help` で help が出ること、exit code 2 を確認
  - `cargo run --quiet --` (引数なし) — トップレベルでも help が出ること確認
  - `cargo run --quiet -- assume xxx -c Json` — alias "Json" が引き続き通り、AWS 呼び出し直前の target 検索で「target(name=xxx) is not found.」が出ることを確認
  - `cargo run --quiet -- assume xxx -c j` — alias "j" も同様
  - `cargo run --quiet -- assume xxx -c bogus` — 不正値が現行と同じ possible values リスト付きで弾かれること確認
- `rm /Users/yuta/space/private/aws-masquerade/src/base/cmd.rs` — Cmd トレイト定義ファイルの削除

## 判断・意思決定

- **ファイル構造を維持**: `cmd/source/list.rs` / `cmd/source/show.rs` のような 1 ファイル 1 コマンド構造は `Cmd` トレイト時代の名残で、derive 化すると中身が極端に短くなる (list.rs は 17 行)。集約も可能だが、kanban 0002 の要望は「clap の書き方を現代化」であり、ファイル構造の再編は別タスクの判断に委ねるべきと判断。差分も git history も最小化できる。
- **`cargo` feature の維持**: `command!()` マクロを全廃するため `cargo` feature は厳密には不要だが、害もないので残した。将来再利用の余地。
- **`CredentialOutputTarget::new(&str)` の維持**: 現状参照がなく dead code だが、削除は「clap の書き方を現代化」の範囲外と判断。CLAUDE.md の「既知の注意点」に dead code である旨を追記。
- **`base::Validation` の維持**: clap と無関係なので維持。
- **`cmd/target/list.rs` の v0/v1 明示分岐の維持**: CLAUDE.md で意図的とされているので、derive 化しても `run` 本体ロジックは触らなかった。
- **`arg_required_else_help` を明示**: derive では `Subcommand` フィールドが非 Option なら `subcommand_required` は暗黙に立つが、`arg_required_else_help` は別物。各 Args 構造体および Cli に明示。動作確認で `aws-masquerade configure` 単独実行が help を出すこと (error にならないこと) を検証。
- **ValueEnum の `name` 上書き**: derive のデフォルト rename は kebab-case で、`PowerShell` → `power-shell`、`SharedCredentials` → `shared-credentials` に化ける。現行挙動を保つため `#[value(name = "PowerShell")]` / `#[value(name = "SharedCredentials")]` を明示。`Json` / `Bash` / `Fish` も対称性のため明示。aliases (`j`, `Json` 等) も `#[value(alias = ..., alias = ...)]` で完全再現。
- **`#[command(about = ...)]` で help テキストを移植**: 現行 builder API で `.about("...")` を使っていた箇所は、derive では `#[command(about = "...")]` (Args 構造体側) または doc comment (Subcommand variant 側) で再現。`Path` の "show config file path" だけは leaf variant のため doc comment、各群の about は Args 構造体側に付与。Subcommand variant 用に Commands enum 内の doc comment は付けない選択 (現行はトップレベルの `aws-masquerade --help` で各群名の右側に about が出るのみ。これは Commands enum variant の doc comment で出るが、現行と完全に一致するよう Commands 内の 4 variant にも doc comment は付けず — と思ったが実際の出力で確認したところ Args 構造体側の about が反映されていた)。最終確認した help 出力は現行と完全一致。

## エラー・問題

実装中に発生した問題はなし。各段階で diagnostics に未解決 import エラーが出たが、すべて「まだ書き換えていないファイルが古い識別子を参照している」状態で、次のステップで解消される予定通りの中間状態。最終 `cargo build` / `cargo clippy --all-targets` ともクリーン (警告ゼロ)。help 出力・引数パース・aliases すべて現行と一致することを動作確認で検証済み。
