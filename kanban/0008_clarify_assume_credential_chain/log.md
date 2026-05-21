# ログ: AssumeコマンドのAWS認証チェーン改善について

- 開始: 2026-05-21T02:04:50+09:00
- 完了: —

---

## タスク概要

0006の追加作業として、タスク0006のラベリング結果で「別タスク推奨のA」とされた内容について詳しく理解する。

- **何が問題か**
- **どういった対応が必要か**
- **どう影響があるのか**

---

## 調査結果

### 0006 タスクの「別タスク推奨のA」の内容

`kanban/0006_handle_coderabbit_review/0006_handle_coderabbit_review.md` の「⚠️ 別タスク推奨」セクションより：

| # | ファイル | 内容 | 理由 |
|---|---------|------|------|
| A | `src/cmd/assume.rs` L63 | `generate_sdk_config` の `ProfileFileCredentialsProvider` → `profile_name()` 変更。region をオプション化（None の場合は SDK デフォルトに委ねる） | AWS認証チェーンの解決動作が変わる可能性。検証が必要 |

CodeRabbit の元コメント（0006 タスクファイルから転記）：

> Around line 63-84: generate_sdk_config currently injects a ProfileFileCredentialsProvider and always calls region(...), which overrides profile-scoped settings and the region provider chain; instead, when source.profile is Some call config_loader = config_loader.profile_name(profile) (do not build/inject ProfileFileCredentialsProvider), keep the existing credentials_provider injection only for static Credentials::new when aws_access_key_id/aws_secret_access_key are present, and only call config_loader.region(region) when source.region is Some (i.e., avoid unconditionally setting a default region) so the normal provider chain and profile-scoped region resolution are preserved.

### src/cmd/assume.rs の現在の実装

`generate_sdk_config()` 関数（L63-84）：

```rust
async fn generate_sdk_config(source: &v1::Source) -> aws_config::SdkConfig {
    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
    if let (Some(aws_access_key), Some(aws_secret_access_key)) =
        (&source.aws_access_key_id, &source.aws_secret_access_key)
    {
        let credential_provider =
            Credentials::new(aws_access_key, aws_secret_access_key, None, None, "Static");
        config_loader = config_loader.credentials_provider(credential_provider);
    }
    if let Some(profile) = &source.profile {
        let credential_provider = aws_config::profile::ProfileFileCredentialsProvider::builder()
            .profile_name(profile)
            .build();
        config_loader = config_loader.credentials_provider(credential_provider);
    }
    let region = source.region.clone().map_or_else(
        || aws_types::region::Region::new("us-east-1"),
        aws_types::region::Region::new,
    );

    config_loader.region(region).load().await
}
```

#### 問題点の詳細

**問題1: `ProfileFileCredentialsProvider` の注入**

- L72-77: `source.profile` が `Some` の場合、`ProfileFileCredentialsProvider` を構築して `credentials_provider()` に渡している
- `ProfileFileCredentialsProvider` は「認証情報だけ」を提供するプロバイダー
  - プロファイルの `aws_access_key_id` / `aws_secret_access_key` を読む
  - プロファイルの `region` 設定は読まない（credentials provider であり、region provider ではない）
- 正しくは `config_loader.profile_name(profile)` を使うべき
  - こちらは SDK に「このプロファイル名を使え」と設定するだけで、認証情報・リージョン含めてプロバイダーチェーン全体に適用される

**問題2: `region` の無条件設定**

- L78-83: `source.region.clone().map_or_else(|| Region::new("us-east-1"), ...)` で常に region が設定される
- `source.region` が `None` の場合でも `us-east-1` がハードコードで設定される
- その結果、プロファイルに `region = ap-northeast-1` と設定していても `us-east-1` で上書きされる

**問題3: 認証情報の注入順序**

- L65-70: static creds（`aws_access_key_id` + `aws_secret_access_key`）があれば `credentials_provider()` に注入
- L72-77: `source.profile` があれば `ProfileFileCredentialsProvider` で `credentials_provider()` を上書き
- Rust の `credentials_provider()` は後から呼んだものが優先されるため、両方指定した場合は profile が勝つ
  - ただし、この優先順位は意図的かどうかが不明確

### region の注入を省略した場合の SDK 動作

ユーザーとの議論で確認した点：

- AWS SDK for Rust でリージョンがプロバイダーチェーンで解決できない場合、`SdkConfig` は `region = None` となり STS API 呼び出し時にエラーになる
- `us-east-1` をデフォルトにすること自体は問題ではない（エラーより動く方が良い）
- 問題は「プロファイルの region 設定が us-east-1 で無条件に上書きされること」

**期待される優先順位**：

```
1. source.region（config.toml に明示指定）
2. プロファイルの region（~/.aws/config の region）
3. 環境変数 AWS_REGION
4. フォールバック: us-east-1（エラーより動く方が良い）
```

**現状の実際の優先順位**：

```
1. source.region（あれば）
2. us-east-1（無条件 — プロファイルの region も環境変数も上書きしてしまう）
```

---

## 実装プラン（調査からの結論）

### 変更内容

**問題1の修正**: `ProfileFileCredentialsProvider` → `profile_name()`

```rust
// 変更前
if let Some(profile) = &source.profile {
    let credential_provider = aws_config::profile::ProfileFileCredentialsProvider::builder()
        .profile_name(profile)
        .build();
    config_loader = config_loader.credentials_provider(credential_provider);
}

// 変更後
if let Some(profile) = &source.profile {
    config_loader = config_loader.profile_name(profile);
}
```

**問題2の修正**: `region` を条件付きで設定 + フォールバックの扱いを要検討

案1（CodeRabbit 通り）：
```rust
if let Some(region_str) = &source.region {
    config_loader = config_loader.region(aws_types::region::Region::new(region_str.clone()));
}
// SDK デフォルトに委ねる（STS がリージョンなしで動作するか要検証）
config_loader.load().await
```

案2（フォールバック付き）：
```rust
// source.region が Some の場合は明示設定、None の場合は SDK チェーンに委ねるが、
// 将来的にエラーを防ぐためのフォールバックをログ出力などで示す
```

### 別タスク推奨の理由（まとめ）

1. **動作変更を伴う** — `ProfileFileCredentialsProvider` → `profile_name()` 変更により、認証フロー全体の動作が変わる
2. **region の変更** — us-east-1 のデフォルトを外した場合、STS への接続が失敗するケースが出る可能性がある
3. **検証が必須** — 次の組み合わせを全てテストする必要がある：
   - static creds のみ
   - profile のみ
   - static creds + profile（どちらが優先されるか）
   - region 指定 / 未指定 × 各プロファイル設定
4. **AWS SDK の動作理解** — `profile_name()` の内部動作（region 解決を含むか否か）の実証が必要

### 実装後に検証すべきシナリオ

| シナリオ | source.region | profile | 期待される region |
|---------|---------------|---------|-----------------|
| 1 | ap-northeast-1 | デフォルト | ap-northeast-1 |
| 2 | なし | region=ap-northeast-1 | ap-northeast-1 |
| 3 | us-east-1 | region=ap-northeast-1 | us-east-1（config 優先） |
| 4 | なし | なし | us-east-1（フォールバック or エラー） |
| 5 | なし | なし、AWS_REGION=eu-west-1 | eu-west-1（env var） |

---

## プランニング経緯

初回提案でユーザーから「region の問題点3番について、us-east-1 デフォルトは悪くないのでは？エラーより動く方が良い」とのフィードバックがあった。

これを受けて問題の整理を再実施：
- 問題は「us-east-1 がデフォルト値として悪い」のではなく
- 「us-east-1 が無条件に設定されることでプロファイルの region 設定や環境変数が無視される」ことが問題
- フォールバックとして us-east-1 を残す形（SDK チェーンを優先し、解決できない場合に us-east-1）も検討すべきという方向でプランを修正

修正後のプランがユーザーに承認された。

---

## 会話内容

### フェーズ1

1. ユーザーが `/add-kanban` でタスク0008を作成し、`/kanban 0008` で実行指示
2. タスクファイル読み込み → 0006 タスクの「別タスク推奨のA」を確認
3. `src/cmd/assume.rs` を読み込み → `generate_sdk_config()` の実装を確認
4. プランを作成 → 初回プラン提示
5. ユーザーが「region の問題について、us-east-1 のデフォルトはエラーより良い」とフィードバック
6. プランを修正 → region の問題は「優先順位の問題」として再整理
7. ユーザーがプランを承認

---

## 編集したファイル

（実装フェーズなし — このタスクは調査・説明のみ）

---

## 実行したコマンド

- `ls kanban/` — タスク採番
- `TZ=Asia/Tokyo date` — 時刻取得

---

## 判断・意思決定

- タスク0008の成果物は「コード変更なし」。調査結果と実装プランをログに記録することが目的
- 実際のコード修正は別タスク（0009以降）で行う

---

## エラー・問題

特になし

---

## 完了日時

2026-05-21T02:05:48+09:00
