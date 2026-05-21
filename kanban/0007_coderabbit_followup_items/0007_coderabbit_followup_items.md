# CodeRabbit レビューの追加対応

## 目的
コードやドキュメントに齟齬があるということなので修正して欲しい。特に4, 8については他に設定していないものがないかも一緒に確認して欲しい。また、今想定している動作に影響がでないかは注意して欲しい。

## 要望
0006の追加対応。対応すべきとされた1~8への対応をして欲しい。

## 完了サマリー

完了日時: 2026-05-21T14:47:00+09:00

CodeRabbit レビュー対応項目 1~8 をすべて実装した。

**修正内容:**
- README.md: 3 項目（L21 Linux プラットフォーム、L114 profile フォールバック、L266 save_totp_counter_history デフォルト値）
- src/cmd/assume.rs: 1 項目（arg_required_else_help 追加）
- src/cmd/configure/mod.rs: 1 項目（validate() を run_migrate に追加）
- src/cmd/source/show.rs: 1 項目（validate() 追加 + Validation import）
- src/cmd/target/show.rs: 1 項目（arg_required_else_help 追加）

**検証:**
- cargo check: ✅ パス
- cargo clippy: ✅ パス
- 動作確認: `cargo run -- assume` / `cargo run -- target show` で help が表示される ✅

項目 7（src/cmd/target/list.rs）は既に load_configuration() 使用で実装済みのためスキップ。
