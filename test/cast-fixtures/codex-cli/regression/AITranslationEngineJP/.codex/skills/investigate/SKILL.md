---
name: investigate
description: Codex 側の設計前調査、探索テスト証跡、修正前調査の作業プロトコル。観測事実、UI 証跡、ログ、未確認事項を根拠 first で扱う判断基準を提供する。
---
# Investigate

## 目的

`investigate` は作業プロトコルである。
`investigator` agent が設計前に必要な証拠、探索テストレーンの探索証跡、修正レーンの修正前調査を集めるための、観測事実、UI 証跡、ログ、仮説、残り 不足 の分け方を提供する。

設計前調査では UI check 専用 skill / agent は置かない。
設計前の UI 根拠 は `investigator` が `investigate` の一部として扱う。

## 対応ロール

- `investigator` が使う。
- 返却先は 呼び出し元 または次 agent とする。
- 担当成果物は `investigate` の出力規約で固定する。
- 探索テストレーンでは担当成果物を `探索証跡` に限定する。
- 修正レーンでは担当成果物を `修正前調査` に限定する。

## 入力規約

- 必須入力: 呼び出し元、investigation_goal、known_context を受け取る。
- 非必須入力: investigation_mode、reproduction_steps、candidate_paths、探索計画、テストデータを受け取る。
- 非必須調査種別: `investigation_mode` は `再現`、`UI 根拠`、`trace`、`リスク報告`、`探索テスト証跡`、`修正前調査` のいずれかを受け取る。
- 必須成果物: active task 文脈 または 呼び出し元提供 investigation 文脈を受け取る。

## 外部参照規約

- エージェント実行定義と実行境界は [investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/investigator.toml) に従う。
- エージェント実行定義: [investigator.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/investigator.toml)
- 実行境界: エージェント実行定義に従う
- `agent-browser` CLI の利用規約は [agent-browser.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/references/agent-browser.md) に従う。
- 探索テストレーンの探索計画は [exploration-test-planning](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/exploration-test-planning/SKILL.md) に従う。
- 探索テスト証跡の雛形は [exploration-test-evidence.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/investigate/assets/exploration-test-evidence.md) とする。
- 探索テスト証跡の task 内 artifact は `docs/exec-plans/active/<task-id>/exploration-test-evidence.md` とする。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。
- 関連 skill: /Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/investigate/SKILL.md

## 内部参照規約

### 拘束観点

- `再現`、`UI 根拠`、`trace`、`リスク報告` の観点
- 観測済み事実、UI 根拠、仮説 の分離
- 探索計画、テストデータ、探索証跡 の分離
- 根拠 path と再現条件の残し方
- 設計を止める 残留リスク の表現

## 判断規約

- 根拠 のない結論を書かない
- 観測事実と仮説を混ぜない
- 設計前の UI 根拠 は `agent-browser` CLI で確認する
- UI 根拠 は画面状態、console、screenshot、操作条件を分けて残す
- 実装 レーン の調査は Codex implementation レーンへ戻す
- 探索テスト証跡は探索計画とテストデータを超えない
- 探索テスト証跡では探索範囲を広げる判断をしない
- 探索テスト証跡では原因仮説を固定しない
- 修正前調査は人間観測、レビュー非通過、検証失敗を超えない
- 修正前調査では実装方針と変更ファイルを確定しない
- 修正前調査では観測事実に基づく影響ファイル候補を返してよい

- observed、UI 根拠、inferred を分ける
- 証跡 path と再現条件を優先する
- 設計継続可否に効く 不足 を残す
- active 規約 は agent に対して 1 ファイルだけ置く。調査種別は selector で扱う。

## 非対象規約

- implementation-scope 承認後の再現、再観測、実装時調査は扱わない。
- 恒久修正、プロダクトテスト追加、implementation レビューは扱わない。
- 承認済み実装範囲や対象 file は確定しない。
- 修正レーンの修正実行入力、レビュー通過根拠は扱わない。
- 設計前調査で UI check 専用 agent を前提にしない。
- 探索計画の作成、バグ一覧の集約、影響ファイルの確定は扱わない。

## 出力規約

- 判断結果: 設計前調査の完了、未完了、停止の判定を返す。
- 根拠参照: 調査判断に使った資料、画面、観測結果を返す。
- 不足情報: 設計判断に不足している項目を返す。
- 次判断材料: 次 agent が判断できる材料を返す。
- 引き継ぎ先: `designer` を返す。
- 渡す対象範囲: 観測済み事実、仮説、残り 不足、残留 risks を返す。
- 調査 mode: 実施した調査の種類を返す。
- 観測事実: 観測済み事実だけを返す。
- UI 証跡: UI を確認した場合は証跡と参照先を返す。
- ログ証跡: ログを確認した場合は証跡と参照先を返す。
- 仮説: 事実と分けて原因候補を返す。
- 観測点: 確認した入口、経路、対象を返す。
- 探索証跡: 探索テスト証跡の場合は探索計画とテストデータに対応する観測事実を返す。
- 修正前調査: 修正レーンの場合は人間観測、レビュー非通過、検証失敗に対応する観測事実、ログ、未確認事項を返す。
- 影響ファイル候補: 修正前調査の場合は観測事実に基づく影響ファイル候補を返す。
- 残り不足: 未確認事項と理由を返す。
- 残留リスク: 設計判断に残る リスク を返す。
- 推奨 next step: 設計継続、追加調査、停止のどれが妥当かを返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコードの変更義務を含めない。

## 完了規約

- 出力規約を満たし、次の 実行者 が再解釈なしで判断できる。
- 不足情報または停止理由がある場合は明示されている。
- 観測事実、UI 根拠、仮説、未観測 不足 を分けた。
- 根拠 path、再現条件、UI check 対象範囲 を残した。
- 探索テスト証跡の場合は、探索計画、テストデータ、観測事実、UI 証跡、ログ証跡、未確認事項を分けた。
- 探索テスト証跡の場合は、`exploration-test-evidence.md` に証跡が記録されている。
- 修正前調査の場合は、人間観測、観測事実、UI 証跡、ログ証跡、未確認事項を分けた。
- design continuation に必要な リスク を返した。
- 必須 根拠: 観測済み事実 根拠, UI 根拠 when mode is UI 根拠, reproduction condition, 根拠 path when used
- 完了判断材料: designer が設計継続か停止かを判断できる。
- 残留リスク: 設計判断に残る リスク が返っている。

## 停止規約

- implementation-scope 承認後の再現や再観測を扱う時
- 恒久修正や プロダクトテスト 追加が必要な時
- implementation レビュー が主目的の時
- 観測条件が不足する場合は停止する。
- 必須入力または必須成果物が不足する場合は停止する。
- 恒久修正が必要なら `designer` へ戻す。
- 実装時調査なら、Codex implementation レーン [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-investigate/SKILL.md) を使う前提で `designer` へ戻す。
- 停止時は不足項目、衝突箇所、戻し先を返す。
- 根拠 なしの結論を書く必要がある場合は停止する。
- 設計前調査で UI check 専用 agent を前提にする場合は停止する。
- implementation-time investigation を扱う場合は停止する。
- 探索計画またはテストデータが不足した探索テスト証跡を扱う場合は停止する。
- 探索テスト証跡で探索範囲を広げる必要がある場合は停止する。
- 修正前調査で実装方針または変更ファイルを確定する必要がある場合は停止する。
- 拒否条件: implementation-time investigation
- 拒否条件: permanent fix request
- 拒否条件: 根拠成果物 不足
