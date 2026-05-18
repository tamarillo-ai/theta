---
name: implementation-scope
description: Codex 側の実装スコープ作業プロトコル。人間レビュー 後に、人間が Codex implementation レーン へ渡せる 引き継ぎ入力 を 承認済み実装範囲、依存、検証単位へ分ける判断基準を提供する。
---
# Implementation Scope

## 目的

`implementation-scope` は作業プロトコルである。
`designer` agent が 人間レビュー 後に、Codex implementation 引き継ぎ入力 を固定するための、分割粒度、依存、検証、完了条件 の見方を提供する。

実行境界、正本、引き継ぎ、停止 / 戻し は [design-bundle](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md) を参照する。

## 対応ロール

- `designer` が使う。
- 呼び出し元は `implement_lane` とする。
- 返却先は `implement_lane` とする。
- 担当成果物は `implementation-scope` の出力規約で固定する。

## 入力規約

- 人間レビュー記録: 承認済みシナリオ設計、承認済み UI 設計、レビュー結果。
- 承認済みシナリオ: 実装範囲の根拠にするシナリオ設計成果物。
- UI 要件契約: UI が関係する場合に参照する UI 設計成果物。
- 承認状態: 呼び出し元が渡す承認済み状態。

## 外部参照規約

- エージェント実行定義と実行境界は [designer.toml](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/agents/designer.toml) に従う。
- 要件正本: [spec.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/spec.md) とする。
- ER 正本: [er.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/er.md) と [diagrams/er](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/diagrams/er/) とする。
- 画面正本: [screen-design](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/screen-design/README.md) とする。
- 上位シナリオ詳細仕様正本: [detail-specs](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/detail-specs/README.md) とする。
- scenario 正本: [scenario-tests](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/scenario-tests/README.md) とする。
- 実装スコープ雛形: [implementation-scope.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implementation-scope/assets/implementation-scope.md)
- Codex implementation レーン 入口: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implement-lane/SKILL.md)
- 実行定義 skill: [SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/design-bundle/SKILL.md)
- 分割参照 architecture: [architecture.md](/Users/iorishibata/Repositories/AITranslationEngineJP/docs/architecture.md) の層構造、transport boundary、依存方向を参照する。
- backend 実装規約: [implement-backend/SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implement-backend/SKILL.md) とする。
- frontend 実装規約: [implement-frontend/SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implement-frontend/SKILL.md) とする。
- 統合境界実装規約: [implement-integration/SKILL.md](/Users/iorishibata/Repositories/AITranslationEngineJP/.codex/skills/implement-integration/SKILL.md) とする。
- 外部成果物 が不足または衝突する場合は停止し、衝突箇所を返す。

## 内部参照規約

### 拘束観点

- `implementation-scope.md` の構成
- 承認済み実装範囲、依存対象、検証コマンド、完了条件
- 検証担当者 判定条件
- 並列実行 判定条件
- secret 本体 と 参照値 の分離条件
- Codex implementation 引き継ぎ入力 の構成
- docs 正本化を 引き継ぎ に混ぜない境界

### 引き継ぎ分割規約

implementation-scope の 引き継ぎ は、トークン量の事前計算ではなく、論理境界と規模の目安で分割する。
1 引き継ぎ は原則として `1 受け入れユースケース × 1 検証意図` に収める。
受け入れユースケースはドメイン名や画面名ではなく、人間またはシステムが開始する処理単位として扱う。

受け入れユースケースとは、1 つの操作またはシステム処理が、永続化、backend 契約、frontend 状態 / UI まで必要範囲を通って成立し、完了後にそのユースケースを原則として再編集しなくてよい単位である。
ただし implementation 引き継ぎ では backend と frontend を同一 引き継ぎ に含めない。
UI が関係する task では frontend 引き継ぎを必須にし、backend 引き継ぎより先に置く。
backend と frontend の分割は原則必須とし、両者を接続する API、Wails、DTO、gateway、adapter 契約は 統合境界 引き継ぎとして別に切る。
公開接点 の固定は独立成果物にせず、backend 引き継ぎ、frontend 引き継ぎ、統合境界 引き継ぎの完了条件へ含める。
シナリオテスト と 単体テスト は実装成果物の完了後に別成果物として切り、依存対象が揃った後に並列実行できる。
APIテスト と UI人間操作E2E は、実装後にシナリオテストとして証明する。
裏側 API、service、検証データ への直接投入は補助検証であり、UI人間操作E2E の完了判定にはしない。

### secret 境界規約

secret を扱う 引き継ぎ は、secret 本体 と 参照値 を分けて書く。
参照値 は UI、DTO、read model に出してよい識別子である。
secret 本体 は provider、外部 API、内部認証 に渡す値である。

secret を扱う 引き継ぎ には次をすべて書く。

- 表示可能参照値: UI、DTO、read model に出してよい参照値。
- secret 本体: provider、外部 API、内部認証 に渡す secret 本体。
- secret 解決責務層: 参照値から secret 本体を解決する責務を持つ層。
- 出力禁止値: log、error summary、audit、要求捕捉、URL、DTO、UI、read model に出してはいけない値。

`credential_ref`、`secret_ref`、`api_key`、`token` などの field 名がある場合は、参照値と secret 本体を同じ値として扱わない。
参照値だけで外部 API へ送信できる形にしない。
secret 本体を DTO、UI、read model、URL、log、error summary、audit、要求捕捉に渡す 完了条件 は書かない。

### 規模判定条件

引き継ぎ を作る前に、既存 code map、類似変更、承認済み実装範囲 からおおよその変更ファイル数と変更行数を見積もる。
変更行数は、生成物、スナップショット、lockfile、docs 正本化を除いた プロダクトコード / プロダクトテストの追加行と削除行の合計として扱う。

規模の目安:

- 通常: `15 files` 以下、かつ `800 changed lines` 以下なら 1 受け入れユースケース 引き継ぎ として扱える
- 注意: `16-25 files` または `801-1500 changed lines` なら、完了条件 が 1 つに閉じ、検証データ が限定できる場合だけ 1 引き継ぎ にしてよい
- 分割必須: `26 files` 以上、または `1501 changed lines` 以上が見込まれるなら、引き継ぎ 前に分割する
- 強制停止: `40 files` 以上、または `2500 changed lines` 以上が見込まれるなら、1 引き継ぎ として渡さず、人間に再計画要求を返す

規模で分割する時は、次の順で切る。

1. 別ユースケースに分けられるならユースケースで切る。
2. 同じ use case 内でも、frontend 実装、backend 実装、統合境界実装は必ず切る。
3. それでも大きい場合は、parse、preview、generation、settings save など 失敗種別 が違う処理で切る。

### 境界規約

import、generation、settings save、preview、create / update / delete、export のように use case が違う処理は、同じ 層 でも分割する。
失敗種別 が違う処理も、可能なら分割する。

同じ受け入れユースケースでも、backend と frontend は 1 引き継ぎ に含めない。
backend 引き継ぎ は永続化、service / usecase、controller、DTO / gateway 境界までを扱う。
frontend 引き継ぎ は承認済み `ui-design.md` を必須根拠にし、状態 / UI を扱う。
統合境界 引き継ぎ は API、Wails、DTO、gateway、adapter 契約 の接続と実画面確認を扱い、backend 実装や frontend UI 実装の代替にしない。

backend 側の 引き継ぎ に含めてよい 層:

- repository / SQLite concrete
- service / usecase
- controller / bootstrap
- gateway 契約 / DTO mapping

frontend 側の 引き継ぎ に含めてよい 層:

- frontend 状態 / presenter / usecase / controller
- frontend UI screen

ただし 注意 以上の規模なら、`補足` に 1 引き継ぎ とする理由、想定変更ファイル数、想定変更行数、分割しない理由を書く。
実行時に通る経路が誤読されやすい場合は、`補足` に `本番経路` として public API / DTO / controller / UI 入口 / persistence path だけを書く。
特定 domain の処理名や業務知識は skill へ持ち込まず、task 内成果物 側へ置く。

禁止例:

- ドメイン名や画面名だけを根拠に複数ユースケースを同じ 引き継ぎ にする
- 通常 / 注意 を超える規模なのに、変更ファイル数と変更行数の見積もりを書かずに 1 引き継ぎ にする
- backend 契約 と frontend UI を同じ 引き継ぎ に含める
- 統合境界 引き継ぎを置かずに backend と frontend の接続を実装引き継ぎへ混ぜる
- migration、import、generation、settings save のような 失敗種別 の違う処理を「同じ画面だから」という理由だけでまとめる

### 並列実行規約

引き継ぎ 作成時は、まず `依存対象` から依存表を作る。
次に、同じ段階で依存が解消できる 引き継ぎ を `実行グループ` にまとめる。
`実行グループ` は `wave-1`、`wave-2`、`wave-3` のように必要な数だけ連番で作る。
`実行グループ` は Codex implementation レーン 側の 着手可能 wave であり、同じ wave 内でも `並列可能対象` に列挙されない 引き継ぎ は並列実行しない。
`ready_wave` は `実行グループ` と同じ値を 引き継ぎ ごとに明示し、着手可能 wave 表で 引き継ぎ 一覧、開始前に完了している依存、並列 pair、阻害要因 を確認できる形にする。

並列実行可能な 引き継ぎ は、次をすべて満たす必要がある。

- `依存対象` が空、または同一実行グループ開始前に完了済みである
- `承認済み実装範囲` の想定変更ファイル / module / test 対象 が他 引き継ぎ と重ならない
- public 契約、DTO、schema、migration、shared 検証データ などの shared 境界 を同時に変更しない
- `検証コマンド` が 引き継ぎ内 で、失敗時に 担当引き継ぎ を特定できる
- UI がある task の frontend 引き継ぎと backend 引き継ぎを同時に開始しない
- 同じ 広域 判定条件 修正や同じ flaky environment 阻害要因 を解消対象にしない

並列不可の task は `並列不可理由` に理由を書く。
理由は `依存対象`、`承認済み実装範囲重複`、`共有契約変更`、`検証担当不明`、`バックエンドフロントエンド順序`、`広域判定条件共有` のいずれかに寄せる。
これ以外の理由が必要な場合は、task 内成果物 側に具体理由を書き、skill 側の共通分類は増やさない。

`実行グループ: wave-1` は即実行可能な 引き継ぎ を指す。
`実行グループ: wave-N` は、`wave-1` から `wave-(N-1)` までのうち、その 引き継ぎ の `依存対象` に必要な 完了条件 が完了した後に実行できる 引き継ぎ を指す。
backend と frontend は別 引き継ぎ のまま維持し、UI がある task では frontend を backend より前の wave に置く。
最終検証、Sonar、Codex レビュー は全 wave 完了後にだけ実行する。

### 初手規約

引き継ぎ 作成時は、各 引き継ぎ に `初手` を必ず書く。
`初手` は Codex implementation レーン が最初に閉じる 1 clause だけを示す。
広い調査開始、複数 clause、partial な advance は書かない。

`初手` には次を含める。

- path
- symbol または対象単位
- 変更種別
- 対応する `完了条件` clause
- 1 手目にする理由

1 edit で clause を閉じられない場合は、同じ clause の最小 closure chain を `補足` または `完了条件` に補足する。
ただし複数 clause を 1 つの `初手` にまとめない。

### 検証担当者判定条件

引き継ぎ 作成時は、各 `検証コマンド` がその 引き継ぎ の 担当者 に属しているかを必ず確認する。
検証 担当者 は、`承認済み実装範囲` の変更だけでその コマンド を 通過 させられる 引き継ぎ である。

各 コマンド は次を満たす必要がある。

- `完了条件` を直接検証している
- `承認済み実装範囲` と解消済み `依存対象` だけで 通過 できる
- 未実装の後続 引き継ぎ を前提にしない
- 失敗した時に、その 引き継ぎ の実装不足として説明できる
- 広域 検証 は原則 `最終検証とレビュー` に寄せる

途中 引き継ぎ に 広域 検証 を置く場合は、広域 コマンド が必要な理由、必須 downstream 対象範囲、分割しない理由を `補足` に書く。
この説明を書けない場合、その コマンド は対象 引き継ぎ の 検証 ではなく 最終検証 に移す。

## 判断規約

- 人間レビュー 後にだけ作る
- 1 引き継ぎ は独立検証可能な粒度にする
- 1 引き継ぎ は原則として `1 受け入れユースケース × 1 検証意図` に収める
- 用語体系は `受け入れテスト > システムテスト > UI人間操作E2E / APIテスト` を正本にする
- `E2E` は UI 人間操作起点だけを指す
- `APIテスト` は 公開接点 起点の システムレベルテスト として扱う
- UI が入口の機能では、裏側の直接呼び出しや 検証データ 直接投入だけを `UI人間操作E2E` の完了条件にしない
- 引き継ぎ が大きいかどうかは、論理境界に加えて想定変更ファイル数と想定変更行数で判定する
- 対象範囲、依存、初手、検証、完了条件 を必ず揃える
- 並列実行可能性は task 出し時に明示する
- 人間レビュー 済みの詳細要求タイプと質問票回答だけを 引き継ぎ根拠にする
- frontend 引き継ぎ は承認済み UI 要件契約を 引き継ぎ根拠にする
- 検証コマンド は 引き継ぎ の 承認済み実装範囲 と 完了条件 だけで 通過 できるものにする
- backend と frontend は必ず別 引き継ぎ に分ける
- UI がある task では frontend 引き継ぎを必須にし、backend 引き継ぎより先に置く
- frontend 引き継ぎ は承認済み UI 要件契約の主要区画、導線、状態表示を維持する完了条件を含める
- 統合境界 引き継ぎ は backend と frontend の間の公開接点、DTO、gateway、adapter 契約を接続する単位として別に作る
- 統合境界 引き継ぎ は UI がある task の実画面確認を完了条件に含める
- シナリオテスト 引き継ぎ と 単体テスト 引き継ぎ は実装成果物の完了後に別成果物として作る
- secret を扱う場合は、参照値、secret 本体、secret 解決責務層、出力禁止値を分ける
- 必要な場合だけ `本番経路` を 補足 に書き、必須 成果物 や domain 固有欄にはしない
- `本番経路` は実行時に通る public API / DTO / controller / UI 入口 / persistence path を指す
- `本番経路` は domain 名や画面名の知識ではなく、引き継ぎ の補助語として扱う
- Codex は承認済み implementation-scope に基づいて 引き継ぎ入力 を作る
- Codex implementation レーン に docs 正本化や 作業流れ 変更を渡さない

- 承認済み 成果物 だけを根拠にする
- 承認済み詳細要求タイプを 検証意図 の根拠にする
- implementation 引き継ぎ を受け入れユースケースで分ける
- downstream 引き継ぎ が依存する 公開接点 を各実装成果物の完了条件として固定する
- 変更ファイル数と変更行数の目安で大きすぎる 引き継ぎ を事前に切る
- 層 をまたぐ時は、完了条件 が受け入れユースケースとして検証できるようにする
- UI人間操作E2E の証明は 実装後の シナリオテスト に寄せる
- 検証コマンド と 完了条件 を揃える
- 初手 を 1 完了条件 clause に固定する
- 検証コマンド が 承認済み実装範囲 と解消済み 依存対象 だけで 通過 できることを確認する
- 並列実行可能な 引き継ぎ は 実行グループ、ready_wave、並列可能対象 で明示する
- 着手可能 wave 表で Codex implementation レーン が読む実行順を先に固定する
- 並列不可の 引き継ぎ は 並列不可理由 に分類済み理由を書く
- 広域 検証 を途中 引き継ぎ に置く場合は、必要な downstream 対象範囲 と理由を 補足 に書く
- backend と frontend を同一 引き継ぎ に入れず、依存対象 で接続する
- backend、frontend、統合境界 の各 引き継ぎ は、implement-lane の対応する 実装成果物 と実装 skill に接続する
- `credential_ref`、`secret_ref`、`api_key`、`token` などの field 名を、参照値と secret 本体の両方に使わない
- `本番経路` が必要な時だけ 補足 に補助情報として書く
- 人間がそのまま `implement_lane` に渡せる 入力にする

## 非対象規約

- 人間レビュー前の実装対象範囲確定は扱わない。
- プロダクトコード実装、実装時の再現、trace、レビュー補助は扱わない。
- Codex から Codex implementation レーンへ直接引き継ぎしない。
- docs 正本化を Codex implementation 引き継ぎに含めない。
- domain 固有知識を skill や雛形の共通例として増やさない。

## 出力規約

- 判断結果: implementation-scope の完了、未完了、停止の判定を返す。
- 根拠参照: 実装範囲の根拠にした承認済み成果物を返す。
- 不足情報: 引き継ぎ入力を固定できない不足項目を返す。
- 次判断材料: `implement_lane` が実装引き継ぎ入力を作れる材料を返す。
- 禁止事項: 出力にツール権限、エージェント実行定義、プロダクトコード変更の指示を含めない。

## 完了規約

- task 内成果物 が承認状態、根拠参照、未決事項を含んでいる。
- 人間レビュー が必要な判断を AI だけで完了扱いにしていない。
- 人間レビュー 承認 を確認した。
- scenario-design に `needs_human_decision` が残っていないことを確認した。
- 承認済み詳細要求タイプを 検証意図 の根拠にした。
- 引き継ぎ を 承認済み実装範囲、依存対象、検証 で分けた。
- 各 引き継ぎ が `1 受け入れユースケース × 1 検証意図` に収まっている。
- 各 検証コマンド が `完了条件` を直接検証している。
- 各 検証コマンド が `承認済み実装範囲` と解消済み `依存対象` だけで 通過 できる。
- 各 引き継ぎ に 1 clause だけを閉じる `初手` を書いた。
- 各 引き継ぎ の想定変更ファイル数と変更行数を見積もった。
- `15 files` / `800 changed lines` 以下を 通常 として扱った。
- `16-25 files` または `801-1500 changed lines` の 注意 引き継ぎ には、1 件にする理由を `補足` に書いた。
- `26 files` 以上または `1501 changed lines` 以上の 分割必須 引き継ぎ を 1 件として渡していない。
- `40 files` 以上または `2500 changed lines` 以上の 強制停止 引き継ぎ は implement-lane へ戻した。
- import / generation / settings save / preview / create / update / delete / export のうち、別 use case になっている処理を同一 引き継ぎ に混ぜていない。
- domain 名や画面名だけを根拠に、複数 use case を同一 引き継ぎ にまとめていない。
- 層 をまたぐ 引き継ぎ は、受け入れユースケース 完了条件 で完了判定できる。
- backend、frontend、統合境界 が必要な場合は別 引き継ぎ として分割されている。
- frontend 引き継ぎ がある場合は、承認済み `ui-design.md` を根拠参照に含めた。
- frontend 引き継ぎ がある場合は、承認済み UI 要件契約の主要区画、導線、状態表示を維持する完了条件を書いた。
- UI がある task では、frontend 引き継ぎを backend 引き継ぎより前の依存対象にした。
- 統合境界 引き継ぎ がある場合は、接続結果の実画面確認を完了条件に書いた。
- シナリオテスト 引き継ぎ と 単体テスト 引き継ぎ は、実装成果物の完了後に並列可能な別成果物として分けた。
- `依存対象` から依存表を作り、着手可能 wave を `実行グループ` と `ready_wave` にした。
- 着手可能 wave 表に 引き継ぎ、開始前依存、並列 pair、阻害要因 を書いた。
- 並列可能な 引き継ぎ だけを `並列可能対象` に列挙した。
- 並列不可の理由を `並列不可理由` に分類済み reason で書いた。
- 並列可能な 引き継ぎ の 承認済み実装範囲、shared 境界、検証 担当者 が重なっていない。
- secret を扱う 引き継ぎ には、表示可能参照値、secret 本体、secret 解決責務層、出力禁止値が分かれて書かれている。
- 広域 検証 を途中 引き継ぎ に置く場合は、必須 downstream 対象範囲 と理由を `補足` に書いた。
- 人間が Codex implementation レーン に渡す 入口、禁止事項、期待完了報告を明示した。

## 停止規約

- 人間レビュー 前に実装 対象範囲 を決める時
- 承認済み implementation-scope なしで `implement_lane` の `backend 実装`、`frontend 実装`、`統合境界実装` へ 引き継ぎ する時
- プロダクトコード を直接実装する時
- 実装時の再現、trace、レビュー 補助を扱う時
- `needs_human_decision` が残る scenario-design から 引き継ぎ を作る必要がある場合は停止する。
- 層だけを根拠に、単体では完了判定できない micro 引き継ぎ を量産する必要がある場合は停止する。
- backend と frontend を同一引き継ぎに含める必要がある場合は停止する。
- 承認済み UI 要件契約がないまま frontend 引き継ぎ を開かない。
- 統合境界 引き継ぎなしに backend と frontend の接続を実装引き継ぎへ混ぜる必要がある場合は停止する。
- UI がある task で frontend 引き継ぎを省略する必要がある場合は停止する。
- UI がある task で backend 引き継ぎを frontend 引き継ぎより先に開始する必要がある場合は停止する。
- UI 入口の引き継ぎで、裏側の直接呼び出しだけを完了条件にする必要がある場合は停止する。
- 変更ファイル数と変更行数が分割必須を超える引き継ぎを 1 件として渡す必要がある場合は停止する。
- 初手がない引き継ぎを Codex implementation レーンに渡す必要がある場合は停止する。
- 初手に複数 clause や曖昧な調査開始を書く必要がある場合は停止する。
- 未実装の後続引き継ぎを必要とする検証コマンドを途中引き継ぎに入れる必要がある場合は停止する。
- 最終検証で見るべき広域コマンドをレーン内検証として扱う必要がある場合は停止する。
- 承認済み実装範囲、shared 境界、検証担当者が曖昧な引き継ぎを並列実行可能として扱う必要がある場合は停止する。
- 同じ実行グループという理由だけで引き継ぎを並列実行する必要がある場合は停止する。
- secret を扱う 引き継ぎ で、参照値、secret 本体、secret 解決責務層、出力禁止値を分けて書けない場合は停止する。
- secret 本体を DTO、UI、read model、URL、log、error summary、audit、要求捕捉に出す 完了条件 が必要な場合は停止する。
- 実装時調査を Codex 再計画前提にする必要がある場合は停止する。
- 停止時は不足項目、衝突箇所、戻し先を返す。
- docs 正本化を Codex implementation レーン 引き継ぎ に混ぜる必要がある場合は停止する。
- 検証コマンドなしの引き継ぎが必要な場合は停止する。
- 未実装の後続 引き継ぎ を必要とする 検証コマンド を途中 引き継ぎ に入れる必要がある場合は停止する。
- 最終検証 で見るべき 広域 コマンド を レーン内検証 として扱う必要がある場合は停止する。
- 同じ `実行グループ` という理由だけで並列実行可能として扱う必要がある場合は停止する。
