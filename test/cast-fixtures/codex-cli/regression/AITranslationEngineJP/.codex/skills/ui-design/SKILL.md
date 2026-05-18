---
name: ui-design
description: Codex 側の UI 設計作業プロトコル。UI 要件契約、実画面確認観点、agent-browser 確認結果を固定する基準を提供する。
---
# UI Design

## 目的

`ui-design` は作業プロトコルである。
`designer` agent が UI を言葉だけで固定せず、UI 要件契約、実画面確認観点、agent-browser 確認結果として扱うための、表示項目、操作、状態差分、導線、主要操作後の画面変化、UX 標準参照の見方を提供する。

実行境界、正本、引き継ぎ、停止 / 戻し は [design-bundle](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md) を参照する。

## 対応ロール

- `designer` が使う。
- 呼び出し元は `implement_lane` または人間とする。
- 返却先は 人間レビュー または `implement_lane` とする。
- 担当成果物は `ui-design` の出力規約で固定する。

## 入力規約

- task 内成果物: UI 要件契約、実画面確認観点、agent-browser 確認結果の根拠にする設計成果物。
- 根拠参照: UI 判断の根拠にする要件、シナリオ、既存画面。
- 承認状態: 呼び出し元が渡す承認済みまたは未承認の状態。

## 外部参照規約

- エージェント実行定義と実行境界は [designer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/designer.toml) に従う。
- 要件正本: [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- architecture 正本: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) とする。
- ER 正本: [er.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/er.md) と [diagrams/er](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/diagrams/er/) とする。
- 画面正本: [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) とする。
- UI 部品アーキテクチャ正本: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) の `UI Component` とする。
- 上位シナリオ詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) とする。
- scenario 正本: [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) とする。
- UX 標準: [UX-standard.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/UX-standard.md) とする。
- UI 設計雛形: [ui-design.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/ui-design/assets/ui-design.md)
- `agent-browser` 利用規約: [agent-browser.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/references/agent-browser.md)
- アプリ起動 command 権限: [default.rules](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/rules/default.rules)
- 実行定義 skill: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md)
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

### 拘束観点

- 画面表示文言、表示項目、主要操作、ボタン有効条件
- 画面区画、状態差分、配置制約、アクセシビリティ
- UI 部品化可否、画面専用部品、共有部品、部品化しない画面構造
- アプリ起動後の実画面 URL、確認 command、実画面と UI 要件契約の差分
- 読み込み中、空、エラー、無効、進行中、再試行、成功
- デスクトップ / モバイルで破綻してはいけない条件と実装後確認観点

### 表示文言変換例

| 固定名または内部状態名 | 画面表示文言 |
| --- | --- |
| `credential missing` | APIキーが未設定です |
| `dirty-validation` | 設定を変更したため、もう一度確認が必要です |
| `getModels failure` | モデル一覧を取得できませんでした |
| `Create job` は無効 | ジョブを作成できません |
| `Ready job` の read-only summary | 作成後の設定内容 |

### UI 部品化判定表

この表は、UI 要件契約で部品化できるものを部品化する判断を拘束する。
UI 部品の正本は `architecture.md` の `UI Component` とする。

| 条件 | 見るもの | 部品化しやすい例 | 分けないほうがよい例 |
| --- | --- | --- | --- |
| 意味が独立している | その部品を一言で説明できるか | `UserStatusBadge`、`SearchForm`、`Pagination` | 右上にある灰色の箱 |
| 入力が明確 | props や引数にできるか | `status`、`label`、`onClick` | 親画面の状態を大量に直接読む |
| 出力が明確 | event や表示結果が限定されるか | `onSubmit(query)`、`onSelect(id)` | 内部で複数の画面状態を勝手に更新する |
| 状態を閉じ込められる | 内部状態と外部状態を分けられるか | 開閉状態、入力中テキスト | 業務フロー全体の進行状態 |
| 変更理由がまとまる | 仕様変更時に同じ理由で変わるか | 日付表示規則、状態表示 | A画面では契約都合、B画面では権限都合で変わる |
| 使用箇所が複数ある | 再利用されるか | ボタン、カード、一覧行 | 1画面専用の大きなレイアウト |
| バリエーションが制御可能 | variant で表現できるか | `primary`、`secondary`、`danger` | props が増えすぎて条件分岐の塊になる |
| テスト単位になる | 単体で期待値を書けるか | `status=pending` なら未確認表示 | 画面全体を起動しないと意味がない |
| デザイン規則を担う | 余白、色、文言規則を統一できるか | `FormField`、`ErrorMessage` | 個別画面の例外スタイル |
| ドメイン概念に対応する | 業務上の概念名を持てるか | `LicenseLimitSummary`、`TenantRoleTable` | 単なる `BoxWithIconAndText` |

## 判断規約

- UI は UI 要件契約で固定し、UI 確認は実画面確認観点として扱う
- UI 部品は `architecture.md` の `UI Component` に従い、画面専用部品と共有部品の二層で判断する
- UI 要件契約では、部品化できる表示単位を部品化対象として残す
- UI 要件契約では、部品化しない表示単位と理由を残す
- 既存画面変更では、既存画面または既存 UI 部品を土台にする
- 既存画面変更では、独自の page shell、card、grid、配色、余白体系を新規に作らない
- 既存画面変更では、変更対象区画だけを差し替え、変更しない区画は既存画面の構造と表示を維持する
- 新規画面では、`docs/screen-design` の画面設計に従う
- `agent-browser` 確認では `docs/references/agent-browser.md` に従い、`open`、`snapshot`、`errors`、`screenshot`、`close` を必要に応じて使う
- 実画面確認が UI 設計根拠に必要な場合は、`designer.toml` の実行境界に従ってアプリを起動し、`agent-browser` で実画面を確認する
- 実画面確認は既存表示、既存導線、既存状態、UI 要件契約との差分を確認するために限る
- 実画面確認でプロダクト不具合または実装時調査が必要になった場合は、`ui-design.md` に根拠を残して `implement_lane` へ戻す
- 汎用的な AI 風 UI や過剰な装飾を要求しない

- UI 契約 と シナリオ の責務を分ける
- デスクトップ と モバイル の破綻条件を実装後確認観点として残す
- 画面表示文言は日本語を優先する
- 画面表示文言は、固定名以外を日本語の業務語へ置き換える
- 内部状態名は画面に出さず、利用者の次操作を示す文へ変換する
- 英語ラベルを画面に出す場合は、利用者が設定画面で見る既存語だけに限定する
- `agent-browser` 確認後に表示文言レビューを必ず行う
- 表示文言レビューは、専門知識がなくても次に何をするか分かる表現水準かを判定する

## 非対象規約

- UI 不要 task、プロダクト frontend 実装、docs 正本反映だけの作業は扱わない。
- プロダクトコード実装と未承認 docs 正本化は扱わない。
- 実装後に人間が確認すべき見た目調整を隠さない。

## 出力規約

- 判断結果: UI 要件契約の完了、未完了、停止の判定を返す。
- 根拠参照: UI 判断の根拠にした要件、シナリオ、既存画面を返す。
- UI 部品化判断: 部品化対象、配置先、分けない対象、判断理由を返す。
- 確認結果: `ui-design.md` の agent-browser 確認結果を返す。
- 実画面確認結果: アプリ起動 command、確認 URL、実画面確認の根拠、UI 要件契約との差分を返す。
- 操作確認結果: 主要操作後の画面変化、状態切り替え、未確認理由を返す。
- 表示文言レビュー結果: `agent-browser` 確認後に行った表示文言レビューの判定を返す。
- 不足情報: UI 要件契約を固定できない不足項目を返す。
- 次判断材料: `designer` または `implement_lane` が次を判断できる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- task 内成果物 が承認状態、根拠参照、未決事項を含んでいる。
- 人間レビュー が必要な判断を AI だけで完了扱いにしていない。
- 表示項目、主要操作、ボタン有効条件を確認した。
- `ui-design.md` は UI 部品化対象、配置先、分けない対象、判断理由を含んでいる。
- `docs/UX-standard.md` の高優先度項目と対象画面に関係する項目を `ui-design.md` に結果付きで残した。
- 状態、状態差分、表示幅追従、はみ出しリスク を実装後確認観点として確認した。
- `ui-design.md` は UI 要件契約と確認観点を含んでいる。
- `ui-design.md` は `agent-browser` で確認した URL、起動 command、人間確認中の起動状態、画面サイズ、UX 標準確認結果、問題、未確認理由を含んでいる。
- 実画面確認を行った場合は、`ui-design.md` にアプリ起動 command、確認 URL、確認した画面、UI 要件契約との差分、未解決事項を含んでいる。
- `ui-design.md` は `agent-browser` 確認後の表示文言レビュー結果を含んでいる。
- 表示文言レビューは、固定名以外の画面表示文言が日本語の業務語になっているかを確認している。
- 表示文言レビューは、内部状態名が画面に出ず、利用者の次操作を示す文へ変換されているかを確認している。
- 表示文言レビューは、英語ラベルが利用者の設定画面で見る既存語だけに限定されているかを確認している。

## 停止規約

- UI が不要で `plan.md` の `ui_design` が `N/A` の時
- プロダクト frontend コードを実装する時
- docs 正本へ UI 仕様を反映するだけの時
- 実画面確認が UI 設計完了に必要で、アプリ起動または `agent-browser` 確認ができない場合は未実行理由を返して停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
