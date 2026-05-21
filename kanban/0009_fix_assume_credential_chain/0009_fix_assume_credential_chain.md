# Assumeコマンドの認証チェーン改善

## 目的
0006の CodeRabbit レビューで「別タスク推奨」とされた認証チェーンの問題を解消する。プロファイルの region 設定や環境変数 `AWS_REGION` がユーザーの期待通りに尊重されるようにする。

## 要望
0008の調査結果に基づき、`src/cmd/assume.rs` の `generate_sdk_config()` を以下の通り修正する:

1. `ProfileFileCredentialsProvider` の注入を `config_loader.profile_name(profile)` に変更する
2. `source.region` が `Some` の場合のみ `config_loader.region()` を呼び、`None` の場合は SDK プロバイダーチェーンに委ねる（フォールバック設計は実装時に決定）

## プラン

`src/cmd/assume.rs` の `generate_sdk_config()` を以下の通り修正する:

1. **`ProfileFileCredentialsProvider` の削除**
   - `config_loader.profile_name(profile)` に置換し、SDK のプロバイダーチェーン全体（認証情報＋region）でプロファイルを参照する

2. **`RegionProviderChain` による region 解決**
   - `first_try(source.region)` → `or_default_provider()` → `or_else("us-east-1")` のチェーンで構築
   - 優先順位: `source.region` → 環境変数/プロファイル → us-east-1 フォールバック

## 完了サマリー

完了日時: 2026-05-21T02:18:31+09:00

`src/cmd/assume.rs` の `generate_sdk_config()` 関数を修正した:

- `ProfileFileCredentialsProvider` を削除し、`config_loader.profile_name(profile)` に置換。プロファイルの region 設定が引き継がれるようになった
- `RegionProviderChain::first_try(source.region).or_default_provider().or_else("us-east-1")` で region 解決チェーンを構築。`source.region`（config.toml）→ 環境変数/プロファイル → us-east-1 フォールバック の優先順位を実現

`cargo build` および `cargo clippy` ともに警告なくクリーンに通過。実 AWS 接続を伴うシナリオ別動作確認は本タスクのスコープ外（実環境でユーザーが確認する想定）。
