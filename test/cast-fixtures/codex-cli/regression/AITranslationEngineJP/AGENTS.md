# AGENTS.md

会話と作業は日本語を基本にする。
英語の key、既存名、command は必要な時だけ使う。

## 日本語出力規約

優先順位は、正確性、可読性、検証可能性、簡潔さの順にする。
技術説明は常体で書く。

## 会話構成

- 質問への直接回答を最初の 1〜2 行で書く。
- 見出しなしの長文を避け、2〜4 個の短い `##` 見出しに分ける。
- 1 段落は 3 文以内、1 文はできるだけ短くする。
- 箇条書きは 3〜6 件に抑え、各項目は 1 論点にする。
- 作業中の進捗報告は 1〜2 文で済ませる。
- 長い出力の末尾には `SUMMARY` を付ける。
- `SUMMARY` は `変更ファイル`、`重要エラー`、`次に見るべき場所`、`再実行コマンド` だけを書く。

## 文体と用語

- 1 文 1 論点にし、主語、対象、作用を省略しすぎない。
- 事実、推測、提案を分け、推測は「可能性がある」と明示する。
- 指示語の使用を原則禁止する。
- 同じ概念には同じ用語を使う。
- ファイル名、key、既存名、command だけを固定名として残す。
- 固定名を残す時は、日本語で意味、状態、理由、影響を補う。

## 説明と図

- 抽象語より具体語を使い、因果は理由、影響、対応のどれかで示す。
- 状態、原因、対応、注意を 1 文に詰め込まない。
- 実装、設計、運用、文書を同じ箇条書きに混ぜない。
- 人間との会話中，順序や依存関係などを説明する場合は必要に応じてチャット本文に ASCII で説明する。
- エージェント間の handoff に ASCII を含めない。

## 報告と自己検査

- 変更報告は 1 行 1 ファイルにする。
- 回答前に、日本語出力規約へ違反している文を書き直す。
- 英語の名詞句は、出典に完全一致で存在する固定名だけ残す。
- 接続詞なしで結論へ飛ばない。
- 根拠のない強い言い切りをしない。
- 不要な横文字、不要な比喩、不要な感情語を入れない。

## 悪い文の判断基準

- 固定名だけを並べ、対象の意味を日本語で説明していない文は悪い文とする。
- 原因、状態、対応、注意を 1 文に詰め込んだ文は悪い文とする。
- 出典にない英語の名詞句を説明用に作った文は悪い文とする。
- 読み手が「何が」「なぜ」「どうなる」を再解釈する必要がある文は悪い文とする。

悪い例:
`body phase candidates は Running cancel と late response rejection を扱っている。`

悪い理由:
`body phase candidates`、`Running cancel`、`late response rejection` の意味を日本語で説明していない。
読み手が対象、状態、動作を推測する必要がある。

良い例:
対象: 本文翻訳段階の候補は、実行中の取り消しを扱う。
対象: 本文翻訳段階の候補は、遅れて返った応答の破棄を扱う。
注意: `Running` のような状態値は、出典にある場合だけ固定名として残す。

## 目的

AITranslationEngineJp は Skyrim Mod 向け翻訳エンジンです。
この repo は agent-first で進めます。

## 参照マップ

最初に使う skill の `SKILL.md` を読む。
agent の 権限 と 契約 は、skill 本文から agent-owned reference を辿る。

- workflow 正本: `.codex/README.md`
- Codex 入口: `.codex/skills/propose-plans/SKILL.md`
- Codex implementation lane 実装入口: `.codex/skills/implementation-orchestrate/SKILL.md`
- 仕様入口: `docs/index.md`
- 長期原則: `docs/core-beliefs.md`
- 恒久要件: `docs/spec.md`
- architecture: `docs/architecture.md`
- 作業計画: `docs/exec-plans/`

## skill 規約の読み方

- `入力規約` は、呼び出し元が渡す情報だけを読む。
- `外部参照規約` は、作業前に必ず確認する skill 外の正本、参照優先順位、衝突時の扱いとして読む。
- 条件付きの外部参照規約は、条件に該当する入力、変更、成果物がある場合に必ず確認する。
- `内部参照規約` は、skill 内の表、分類、依存関係が判断を拘束する条件として読む。
- `判断規約` と `非対象規約` は、判断してよい範囲と扱わない範囲として読む。
- `出力規約` は、返す成果物、置き場所、粒度として読む。
- `完了規約` と `停止規約` は、閉じてよい条件と人間へ戻す条件として読む。

## 強い制約

- Codex は設計、計画、handoff、docs 正本化を担当する
- Codex implementation lane は承認済み `implementation-scope` から実装する
- AI design review は行わず、人間が design bundle を review する
- 設計判断は AI 駆動を前提にする
- Codex implementation lane は `docs/`、`.codex/`、`.codex/skills`、`.codex/agents` を変更しない
- docs 正本化は Codex の `updating-docs` だけが扱う
- 差分量を最小化してはならない。
- 正しい責務境界の中で、必要最小の変更範囲を選ぶ。
- 局所修正は、対象の振る舞いが本当に局所的であると説明できる場合だけ許可する。

## 補足

- library の書き方は `npx ctx7 library` / `npx ctx7 docs` で Context7 を確認する
- wails は `npm run dev:wails:agent-browser` で起動する
- ブラウザ操作は `agent-browser` CLI を使う
- UI 証跡は `agent-browser open http://localhost:34115` から取得する
- Sonar project は `ishibata91_AITranslationEngineJP`
