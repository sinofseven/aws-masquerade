# aws-masquerade (branch: v1-master)

- 概要
  - AWSのAssumeRole用のCLIツール
  - v1を目指して全面改修する
    - コマンド体系も変える
    - 設定ファイルも変える
      - 中身の構造も変える
        - `account`と一つにまとめていたものを、AssumeRoleを実行する `source`と、AssumeRoleの対象となる`target`に分ける
      - jsonからtomlに変える
- コマンド体系
  - [x] configure: 設定ファイルに関するコマンド
    - [x] path: 設定ファイルのパスを表示
    - [x] validate: 設定ファイルに不備がないか確認
    - [x] migrage: 設定ファイルをv0からv1に更新
  - [ ] source: AssumeRoleを実行する側の設定に関するコマンド
    - [ ] list: 一覧を表示する
    - [ ] show: 詳細を表示する
    - [ ] add: 追加する
    - [ ] edit: 設定を変更する
    - [ ] remove: 設定を削除する
  - [ ] target: AssumeRoleの対象の設定に関するコマンド
    - [ ] list: 一覧を表示する
    - [ ] show: 詳細を表示するう
    - [ ] add: 追加する
    - [ ] edit: 設定を変更する
    - [ ] remove: 設定を削除する
  - [ ] assume: AssumeRoleを実行する

### Author
- [sinofseven](https://github.com/sinofseven)
