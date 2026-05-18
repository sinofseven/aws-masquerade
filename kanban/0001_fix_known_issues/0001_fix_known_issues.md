# 既知の問題点を修正

## 目的
問題をなくしたい

## 要望
既知の問題点を修正してください

## 完了サマリー

**完了日時:** 2026-05-18T15:28:00+09:00

### 修正内容

#### 1. variables.rs の定数名を修正 (src/variables.rs:35)
- `pub const REMOTE: &str = "remove";` → `pub const REMOVE: &str = "remove";`
- 理由: `source::sub_command` の命名規則と統一

#### 2. Configuration::validate のエラーメッセージのタイポを修正 (src/models/configuration/v1.rs)
- 180行目: `tource` → `source`、`dupplicate` → `duplicate`
- 187行目: `tource` → `target`、`dupplicate` → `duplicate`（エラーメッセージを「target name」に統一）

### 検証結果
- ✓ `cargo check`: 成功
- ✓ `cargo clippy`: 成功
- ✓ 修正内容を grep で確認

すべての修正が正常に適用され、検証も完了した。
