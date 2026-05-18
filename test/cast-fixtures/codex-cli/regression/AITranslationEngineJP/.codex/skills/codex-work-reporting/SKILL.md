---
name: codex-work-reporting
description: Codex 側の run 全体レポート作業プロトコル。Codex / Codex implementation レーン の完了根拠、レビュー最終状態 YAML、改善ログ から work_history レポート と次回改善事項を残す判断基準を提供する。
---
# Codex Work Reporting

## 目的

`codex-work-reporting` は作業プロトコルである。
Codex 作業流れ の完了、停止、戻し時に、`work_history` へ残す run 全体レポート 材料を整理する。
Codex と Codex implementation レーン の完了根拠、レビュー最終状態、改善ログ、検証結果 を同じ run 単位で集約する。
改善可能性の抽出は、完了根拠 と `workflow-improvement-log.jsonl` を主材料にする。
work_reporter は完了根拠、レビュー最終状態 YAML、改善ログ、検証結果だけを前提にする。

この skill は実行主体ではない。
実行境界は参照元 agent TOML に従い、完了条件と停止条件は参照元 skill に従う。

## 対応ロール

- 呼び出し元は 終了処理、停止、戻しを扱う Codex agent とする。
- 返却先は人間と `work_history` レポート とする。
- 担当成果物は `codex-work-reporting` の出力規約で固定する。

## 入力規約

- run 対象: run 全体レポートを作る `work_history/runs/YYYY-MM-DD-<task-id>-run/`。
- 完了根拠: run 対象で確認できる完了、停止、戻し の根拠。
- 会話ログ参照一覧: run 対象の `transcript_refs.json` または未作成理由。
- 改善ログ: run 対象の `workflow-improvement-log.jsonl` または未作成理由。
- 検証結果: run 対象で確認できる検証結果または未実行理由。

## 外部参照規約

- 作業履歴規約: [README.md](/Users/iorishibata/Repositories/AITranslationEngineJP/work_history/README.md)
- run index 雛形: [README.md](/Users/iorishibata/Repositories/AITranslationEngineJP/work_history/templates/run/README.md)
- Codex レポート 雛形: [codex.md](/Users/iorishibata/Repositories/AITranslationEngineJP/work_history/templates/run/codex.md)
- 実行定義 agent: [work_reporter.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/work_reporter.toml)
- エージェント実行定義と実行境界は [work_reporter.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/work_reporter.toml) に従う。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。
- 関連 skill: /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/codex-work-reporting/SKILL.md

## 内部参照規約

### 拘束観点

- `work_history/templates/run/README.md` の run 全体 要約
- `work_history/templates/run/codex.md` の記入観点
- `transcript_refs.json` の runtime と transcript_path
- `docs/exec-plans/<state>/<task-id>/reviewback.*.yaml` のレビュー最終状態と改善可能性
- `work_history/runs/<run>/workflow-improvement-log.jsonl` の改善ログ
- 改善、時間、無駄、困りごとの分離
- Codex 固有の設計、人間介入、引き継ぎ、正本化判断の記録
- Codex implementation レーン 固有の 完了済み引き継ぎ、変更ファイル、検証、残留 の記録

### 作業観測

作業観測は次回改善用の材料である。
速度の閾値や観測値欠落を初期終了判定には使わない。
work_reporter はスコア算出を行わない。
既存の観測値が run 対象にある場合は、作成済み根拠として扱う。

## 判断規約

- `work_reporter` は最後に必ず run 全体レポート を作る。
- 置き場所は `work_history/runs/YYYY-MM-DD-<task-id>-run/` に固定する。
- 改善可能性の抽出は完了根拠、会話ログ参照、`workflow-improvement-log.jsonl` を主材料にする。
- レビュー最終状態は `reviewback.*.yaml` を正本にする。
- work_history 側に観点別の非通過 YAML は作らない。
- 残留指摘は `reviewback.*.yaml` の `must_fix_open`、`max_level`、`review_status` から観点別に要約する。
- 改善ログは構造問題、作業流れ問題、権限問題、実行問題、人間フィードバック、レビュー由来の改善示唆を抽出する主材料にする。
- `README.md` は人間向け run 全体レポート にする。
- `codex.md` は `work_reporter` が 根拠 から生成する。
- 事実と判断材料を分ける。
- 分からない項目は `未確認`、`不明`、`なし` のいずれかで明示する。
- Codex implementation レーン 側の実装事実は、run 内レポート、完了根拠、レビュー最終状態 YAML、改善ログ から確認できる範囲だけ転記する。
- レビュー最終状態 YAML 欠落、改善ログ欠落、完了根拠不足、会話ログ参照不足 は次回改善 指摘 として扱う。
- 速度指標は改善観測であり、初期終了判定には使わない。
- `.codex/history` には触れず、`work_history/` を使う。
- レポートは次回の指示、引き継ぎ、雛形 改善へ戻せる粒度にする。

- `work_reporter` で run 全体レポート を作る
- `work_history/runs/YYYY-MM-DD-<task-id>-run/` を唯一の レポート 置き場所にする
- `transcript_refs.json` を会話ログ参照の正本として扱う
- `reviewback.*.yaml` を レビュー最終状態の正本として扱う
- `workflow-improvement-log.jsonl` から改善可能性を抽出する
- Codex が実際に見た 根拠 と推測を分ける
- Codex implementation レーンの事実は run 内レポート、完了根拠、レビュー最終状態 YAML、改善ログ からだけ扱う
- 人間が次に見るべきパスや コマンド を残す
- 重要エラーと未実行 検証 を短く明示する

## 非対象規約

- プロダクトコード、プロダクトテスト、docs 正本化は扱わない。
- docs 正本化の承認、対象範囲、implementation-scope を代替しない。
- `docs/exec-plans/`、`.codex/history/`、引き継ぎファイルを run レポート置き場にしない。
- 速度指標を初期終了判定に使わない。
- スコア算出を実行しない。

## 出力規約

- 判断結果: run 全体レポートの完了、未完了、停止の判定を返す。
- 根拠参照: レポート生成に使った完了根拠、会話ログ参照、レビュー最終状態、改善ログ、検証結果を返す。
- 不足情報: レポート生成を完了できない不足項目を返す。
- 次判断材料: 人間または `implement_lane` が次を判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。
- レポートパス: README.md、codex.md、transcript_refs.json、reviewback.*.yaml、workflow-improvement-log.jsonl のパスまたは未作成確認を返す。
- レビュー正本一覧: 観点別の `reviewback.*.yaml` の参照先を返す。
- 観点別残留要約: 観点別の最終状態、未解決指摘の有無、最大重大度を返す。
- 改善ログ要約: 改善ログの分類別件数、重大度、次回改善候補を返す。
- 作業観測 summary: 完了根拠、会話ログ参照、改善ログから分かる範囲で、時間がかかったこと、無駄だったこと、困ったことを返す。
- Codex レポート summary: run 内レポート、完了根拠、レビュー最終状態 YAML、改善ログ から確認できる結果、未完了、重要エラー、検証不足、次に見るべき場所を返す。
- Codex implementation レーン レポート summary: run 内レポート、完了根拠、レビュー最終状態 YAML、改善ログ から確認できる完了 引き継ぎ、変更ファイル、検証結果、統合 レビュー 結果、残留リスク、次に見るべき場所を返す。
- run 全体 指摘: 改善すべきこと、時間がかかったこと、無駄だったこと、困ったこと、検証で不足したことを返す。
- 作業観測 品質 指摘: 完了根拠、会話ログ参照、レポート、実行定義 の欠落または破損を次回改善事項として返す。
- 次回改善: 指示、引き継ぎ、雛形、作業観測の改善を返す。
- 残留 不足: 未確認、不明、なしを区別して返す。

## 完了規約

- 出力規約を満たし、次の 実行者 が再解釈なしで判断できる。
- 不足情報または停止理由がある場合は明示されている。
- `work_reporter` が run 全体レポート を作った。
- `work_history/templates/run/README.md` の必須項目を確認した。
- `work_history/templates/run/codex.md` の必須項目を確認した。
- `transcript_refs.json` を会話ログ参照の正本として扱った。
- `reviewback.*.yaml` を確認し、レビュー最終状態を観点別に要約した。
- `workflow-improvement-log.jsonl` を確認し、改善可能性を分類別に要約した。
- 改善可能性は完了根拠、会話ログ参照、改善ログから抽出した。
- 改善、時間、無駄、困りごとを分けた。
- 人間介入、引き継ぎ、docs 正本化判断を記録対象にした。
- implementation レーンの事実を run 内レポート、完了根拠、レビュー最終状態 YAML、改善ログ からだけ扱った。
- 必須根拠として、完了根拠または不足理由、レビュー最終状態 YAML または不足理由、改善ログまたは未作成理由、レポート 雛形 paths、利用可能な 検証結果 がある。
- 完了判断材料として、work_history/runs/<run>/README.md と codex.md が 根拠 から生成され、次回改善事項が明示されている。
- 残留リスクとして、未確認または不明な 不足 が返っている。

## 停止規約

- プロダクトコードまたはプロダクトテスト を変更する時
- Codex implementation レーン 側の実装事実を推測で補う時
- docs 正本化の承認や 対象範囲 を代替する時
- 速度の数値閾値で終了可否を判定する時
- スコア算出を必要条件にする時
- 停止時は不足項目、衝突箇所、戻し先を返す。
- run 対象が不足する場合は停止する。
- transcript_refs、レビュー最終状態 YAML、改善ログ、完了根拠 の有無を確認できない場合は停止する。
- レポート書き込み先が `work_history/runs` 外になる場合は停止する。
- implementation レーンの事実と推測を区別できない場合は停止する。
- 必須レポートパスを特定できない場合は停止する。
- レポート生成にプロダクトまたは docs 正本の変更が必要な場合は停止する。
