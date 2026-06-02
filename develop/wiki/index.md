おすすめは、**普通のWikiではなく「数学知識グラフを中心にした、多言語・証明付きMathWiki」** として設計することです。

最初から「ページ」を中心にすると、あとで多言語化、定理依存関係、形式証明、同じ定理の別証明、別基礎論理での証明、記法違いを扱うのが苦しくなります。中心に置くべきものはページではなく、**定義・定理・証明・概念・記法・文献・形式証明証明書のエンティティ**です。

一言で言うと、目指すべき姿はこれです。

```text
数学Wiki = 多言語記事 + 数学知識グラフ + 証明データベース + 形式証明レジストリ
```

---

# 1. 参考にすべき既存プロジェクト

既存例から学ぶべき点は多いです。

ProofWiki は自らを「mathematical proofs の online compendium」と説明し、証明の収集・共同編集・分類を目的にしています。2026年3月時点のトップページ表示では、29,443 proofs と 35,139 definitions が示されています。これは「証明中心Wiki」が成立することを示す良い先例です。([ProofWiki][1])

Stacks Project は、代数幾何のためのオープンソースな教科書兼リファレンスで、オンライン閲覧・検索・ハイパーリンクによって依存する補題や定理を辿れる設計を重視しています。また、結果に恒久的なタグを付けると説明しています。これは数学サイトにおける **安定ID** の重要性を示しています。([The Stacks Project][2])

Kerodon も Stacks Project 的なタグ方式を採り、定義・補題・定理・命題・例・節・式などに安定タグを与え、内容が移動しても同じ数学的対象を指し続ける設計を説明しています。これは、あなたのサイトでも必須です。([Kerodon][3])

Mathlib は Lean のコミュニティ駆動の形式化数学ライブラリで、Lean公式サイトは over two million lines of formalized mathematics と説明しています。つまり、長期的には非形式的な解説Wikiだけでなく、形式証明ライブラリとの接続を前提にした方がよいです。([Lean Language][4])

多言語と構造化データについては、Wikibase と MediaWiki Translate が参考になります。Wikibase は共同編集可能な情報を保存・整理し、機械が消費しやすく、多言語化や Linked Open Data として共有しやすい仕組みとして説明されています。([MediaWiki][5]) MediaWiki の Translate 拡張は、wiki内翻訳・校正・翻訳メモリ・機械翻訳支援などを提供します。([MediaWiki][6])

---

# 2. 最重要設計方針

最初に決めるべき原則はこれです。

```text
ページ中心ではなく、エンティティ中心にする。
```

悪い設計：

```text
/ja/自然数の加法の単位元
/en/Additive_identity_of_natural_numbers
/fr/...
```

これだと、各言語ページが別々の実体になり、同じ定理を指しているのか、微妙に違う主張なのか管理できなくなります。

望ましい設計：

```text
Entity: T00001234
type: theorem
canonical_name: Nat.add_zero
formal_statement: ∀ n : Nat, n + 0 = n

labels:
  ja: 自然数の加法における右単位元
  en: Right identity of addition on natural numbers
  fr: Élément neutre à droite de l’addition des entiers naturels

pages:
  ja: /ja/theorem/T00001234
  en: /en/theorem/T00001234
  fr: /fr/theorem/T00001234
```

つまり、**数学的対象は1つ、表示と言語は複数**にします。

---

# 3. サイトの基本構造

最終形では、次の4層に分けるのがよいです。

```text
[4] 多言語Web表示層
    記事、検索、閲覧、翻訳、コメント、レビュー

[3] 数学知識グラフ層
    定義・定理・証明・依存関係・文献・分類

[2] 証明・形式化層
    人間向け証明、Lean/Rocq/NPA証明、証明証明書

[1] 永続ID・バージョン・監査層
    entity ID、hash、履歴、ライセンス、レビュー状態
```

この分離が重要です。
Webページは後から作り直せますが、**数学的エンティティID、定理依存関係、証明履歴、形式証明対応**は後から直すのが非常に大変です。

---

# 4. エンティティ設計

## 4.1 エンティティ種別

最低限、次の型を持たせます。

```text
Concept
  数学的概念。例: 群、位相空間、連続写像

Definition
  定義。例: 群の定義、コンパクト性の定義

Theorem
  定理・命題・補題・系

Proof
  人間向け証明

FormalProof
  Lean / Rocq / NPA などの形式証明

Example
  例

Counterexample
  反例

Notation
  記法

Construction
  構成。例: 商群、テンソル積、直積

TheoryContext
  前提となる理論・基礎・公理系

Reference
  文献

Person
  数学者・著者

Topic
  分野・分類

Problem
  未解決問題・演習問題
```

すべてに安定IDを振ります。

```text
C00000001  Concept
D00000001  Definition
T00000001  Theorem
P00000001  Proof
F00000001  FormalProof
N00000001  Notation
R00000001  Reference
```

Stacks Project や Kerodon のようなタグ方式から学ぶべきなのは、数学的対象に対する**恒久参照**を最初から設計に入れることです。([The Stacks Project][2])

---

# 5. 定理ページの理想構造

定理ページは、単なる文章ページではなく、構造化された情報を持つべきです。

例：

```text
T00001234: Nat.add_zero
```

ページ構成：

```text
1. 名前
   日本語名、英語名、別名、記号名

2. 主張
   人間向け文
   形式的文
   使用する定義
   前提条件

3. 文脈
   基礎理論
   対象分野
   必要な前提
   公理依存

4. 証明
   証明A: 初等的証明
   証明B: 帰納法による証明
   証明C: 代数構造からの一般化

5. 形式証明
   Lean
   Rocq
   NPA
   証明証明書 hash
   使用axiom
   import version

6. 関連項目
   一般化
   特殊化
   系
   逆
   類似定理
   反例

7. 依存関係
   この定理が使う定義・補題
   この定理を使う定理

8. 文献
   初出
   教科書
   論文
   参考URL

9. 多言語
   各言語の翻訳状態
   用語対応
   翻訳レビュー状況
```

---

# 6. 定理データモデル

たとえば、定理はこう表します。

```json
{
  "id": "T00001234",
  "type": "Theorem",
  "canonical_name": "Nat.add_zero",
  "labels": {
    "ja": "自然数の加法における右単位元",
    "en": "Right identity of addition on natural numbers"
  },
  "aliases": {
    "ja": ["n + 0 = n"],
    "en": ["addition by zero"]
  },
  "statement": {
    "informal": {
      "ja": "任意の自然数 n について、n + 0 = n である。",
      "en": "For every natural number n, n + 0 = n."
    },
    "formal_latex": "\\forall n \\in \\mathbb{N},\\ n + 0 = n",
    "formal_ast": {
      "language": "NPA-Core",
      "hash": "sha256:..."
    }
  },
  "context": {
    "foundation": "ConstructiveTypeTheory",
    "requires": ["D00000001"],
    "axioms_allowed": []
  },
  "classification": {
    "msc": ["03E", "11A"],
    "topics": ["NaturalNumbers", "Arithmetic"]
  },
  "proofs": ["P00004567", "P00004568"],
  "formal_proofs": ["F00000091"],
  "dependencies": ["D00000001", "T00000002"],
  "used_by": ["T00002001", "T00002002"],
  "status": "formally_verified"
}
```

Wikidata/Wikibase の発想に近く、概念や対象を item として持ち、statement を property-value として記録し、必要なら qualifier・reference・rank で文脈づける設計が向いています。Wikidata の statement は item についての property-value pair を基本にし、qualifier や reference で文脈づけられると説明されています。([Wikidata][7])

---

# 7. 証明モデル

証明は定理と分離します。

理由は、1つの定理に複数の証明があり得るからです。

```text
T00001234
  ├── P00000001: 帰納法による証明
  ├── P00000002: モノイドの一般定理からの証明
  └── P00000003: 形式証明から生成した証明
```

証明データ：

```json
{
  "id": "P00000001",
  "type": "Proof",
  "proves": "T00001234",
  "method": ["induction"],
  "difficulty": "beginner",
  "language_neutral_steps": [
    {
      "id": "s1",
      "kind": "intro",
      "uses": []
    },
    {
      "id": "s2",
      "kind": "induction",
      "on": "n"
    },
    {
      "id": "s3",
      "kind": "rewrite",
      "uses": ["D00000001"]
    }
  ],
  "localized_text": {
    "ja": "...",
    "en": "..."
  },
  "dependencies": ["D00000001", "T00000002"],
  "status": "reviewed"
}
```

証明本文も翻訳対象にしますが、**証明ステップIDは言語非依存**にします。これにより、ある言語だけ証明ステップが抜ける問題を検出できます。

---

# 8. 形式証明との接続

最終的に「すべての数学の定理」を目指すなら、形式証明への接続は最初から入れるべきです。

形式証明エンティティ：

```json
{
  "id": "F00000091",
  "type": "FormalProof",
  "proves": "T00001234",
  "system": "Lean",
  "system_version": "4.x",
  "library": "Mathlib",
  "formal_statement": "theorem Nat.add_zero ...",
  "proof_code_ref": "git:...",
  "certificate_hash": "sha256:...",
  "kernel_checked": true,
  "axioms_used": [],
  "imports": [
    {
      "module": "Mathlib.Data.Nat.Basic",
      "hash": "sha256:..."
    }
  ],
  "status": "verified"
}
```

大事なのは、形式証明を「参考リンク」ではなく、定理エンティティに紐づく検証済み証拠として扱うことです。Mathlib は形式化数学の大規模ライブラリとして成長しており、こうした外部形式ライブラリとの対応表を持つことは長期的に大きな価値があります。([Lean Language][4])

ただし、形式証明が存在しない定理も大量にあります。したがって、ページの状態は段階的にします。

```text
draft
  下書き

informal
  非形式的な定理文・証明あり

reviewed
  人間レビュー済み

formalized
  Lean/Rocq/NPA等に形式化済み

verified
  形式証明が kernel/checker で検証済み

certified
  独立checker・certificate hash・axiom report まで確認済み
```

---

# 9. 多言語設計

多言語化は、後付けではなく最初から設計に入れるべきです。

## 9.1 言語非依存ID

すべての数学対象は言語非依存IDを持ちます。

```text
T00001234
```

各言語ページは、そのIDの表示です。

```text
/ja/T00001234
/en/T00001234
/fr/T00001234
/de/T00001234
```

URLはSEO向けにslugを足してもよいです。

```text
/ja/theorem/T00001234/自然数の加法における右単位元
/en/theorem/T00001234/right-identity-of-addition-on-natural-numbers
```

slugは変更可能、IDは不変にします。

## 9.2 翻訳単位

ページ全体を1つの翻訳対象にしない方がよいです。
数学記事は構造化されているので、翻訳単位を分けます。

```text
title
short description
statement
intuition
proof step 1
proof step 2
proof step 3
examples
notes
references
```

MediaWiki Translate は、wiki内翻訳、校正、翻訳メモリ、機械翻訳支援、未使用パラメータ警告などを提供すると説明されています。これと同様に、翻訳単位・翻訳メモリ・校正ワークフローを標準機能にするのがよいです。([MediaWiki][6])

## 9.3 用語辞書

数学では、翻訳の一貫性が非常に重要です。

例：

```text
field
  ja: 体
  fr: corps
  de: Körper

ring
  ja: 環
  fr: anneau
  de: Ring
```

用語辞書をエンティティ化します。

```json
{
  "id": "C00000123",
  "canonical_name": "Field",
  "labels": {
    "ja": "体",
    "en": "field",
    "fr": "corps"
  },
  "disambiguation": [
    {
      "term": "field",
      "meaning": "algebraic field"
    },
    {
      "term": "field",
      "meaning": "vector field"
    }
  ]
}
```

「field」のように英語では曖昧で、日本語では別語になる概念は多いです。
単なる翻訳テーブルではなく、**概念IDに紐づく用語辞書**にするべきです。

## 9.4 翻訳状態

各言語ごとに状態を持たせます。

```text
missing
machine_draft
human_translated
reviewed
mathematically_reviewed
outdated
```

特に重要なのは `outdated` です。
英語版の定理文や証明が更新されたら、日本語版などに自動で「原文更新後未確認」と出します。

```json
{
  "entity": "T00001234",
  "language": "ja",
  "translation_status": "outdated",
  "source_revision": "rev_100",
  "translated_from_revision": "rev_093"
}
```

---

# 10. 数学的文脈の設計

数学の定理は、文脈なしには意味が決まりません。

例：

```text
連続
```

は少なくとも次の文脈があります。

```text
位相空間の間の連続写像
距離空間の間の連続写像
実関数のε-δ連続性
順序位相における連続性
```

したがって、定義・定理には必ず `Context` を持たせます。

```json
{
  "id": "CTX000012",
  "type": "TheoryContext",
  "name": "TopologicalSpaces",
  "assumptions": [
    "X : TopologicalSpace",
    "Y : TopologicalSpace"
  ],
  "foundation": "ConstructiveTypeTheory",
  "classical": false,
  "choice": false
}
```

定理側：

```json
{
  "theorem": "T00004567",
  "context": "CTX000012",
  "statement": "A map f : X -> Y is continuous iff ..."
}
```

これにより、同じ日本語名でも異なる概念を安全に扱えます。

---

# 11. 依存関係グラフ

このサイトの最大の価値は、定理同士の依存関係を可視化できることです。

エッジ例：

```text
Definition uses Definition
Theorem uses Theorem
Theorem uses Definition
Proof proves Theorem
FormalProof verifies Theorem
Theorem generalizes Theorem
Theorem specializes Theorem
Theorem equivalent_to Theorem
Theorem has_counterexample Counterexample
Theorem depends_on_axiom Axiom
```

例：

```text
Nat.zero_add
  uses Nat.rec
  uses Eq.refl
  implies Nat.add_left_identity
  used_by Nat.add_comm
```

依存関係グラフがあると、次ができます。

```text
- この定理を理解する前に読むべき項目
- この定理がどこで使われるか
- 証明の依存公理
- 定理の一般化・特殊化
- AIのpremise retrieval
- 学習コース自動生成
- 不備のある証明依存の検出
```

---

# 12. ページ表示の設計

1ページにすべてを詰め込むより、タブ分けがよいです。

```text
概要
  定理文、直感、図、最小限の説明

証明
  複数証明、証明ステップ、依存補題

形式証明
  Lean/Rocq/NPAコード、certificate hash、axiom report

依存関係
  prerequisites, used by, generalizations

例・反例
  examples, non-examples, counterexamples

文献
  books, papers, historical notes

翻訳
  language status, terminology, translation diffs

編集履歴
  revisions, reviewers, discussions
```

数学の利用者はレベルが大きく異なるので、説明レイヤーも分けます。

```text
Intuition
  直感的説明

Standard proof
  標準的な証明

Detailed proof
  省略の少ない証明

Formal proof
  機械検証済み証明

Research notes
  発展的コメント
```

---

# 13. 検索設計

普通の全文検索だけでは不十分です。

必要な検索は次です。

```text
1. キーワード検索
   "compact", "加法 単位元"

2. 数式検索
   "x + 0 = x"

3. 型・構造検索
   "?x + 0 = ?x"

4. 定理検索
   現在のgoalに使える定理を探す

5. 依存関係検索
   この定理を使う定理

6. 分野検索
   代数、解析、位相、圏論

7. 文献検索
   著者、書籍、論文

8. 多言語検索
   日本語で検索して英語項目も出す

9. 形式証明検索
   Lean名、Rocq名、NPA entity ID
```

検索インデックスには、次を入れます。

```json
{
  "entity_id": "T00001234",
  "type": "Theorem",
  "labels": {
    "ja": ["自然数の加法における右単位元"],
    "en": ["Right identity of addition on natural numbers"]
  },
  "symbols": ["+", "0", "="],
  "formal_patterns": ["?n + 0 = ?n"],
  "dependencies": ["Nat", "Nat.add", "Nat.zero"],
  "topics": ["Arithmetic", "NaturalNumbers"],
  "proof_status": "verified"
}
```

---

# 14. 技術スタック案

## 14.1 推奨アーキテクチャ

長期的に考えるなら、次の構成をおすすめします。

```text
Frontend:
  Next.js / SvelteKit / Nuxt
  多言語ルーティング、MathJax/KaTeX、図、検索UI

Backend API:
  Rust / TypeScript / Go
  entity API、proof API、search API、translation API

Primary DB:
  PostgreSQL
  entity, revision, translation, permission, review state

Graph:
  PostgreSQL recursive query から開始
  後で Neo4j / RDF store / custom graph index も検討

Search:
  OpenSearch / Meilisearch / Typesense
  数式検索は専用indexを追加

Object storage:
  proof certificates, formal proof artifacts, generated PDFs

Version control:
  Git-like revision model
  重要データはcontent-addressed hash付き

Formal proof workers:
  Lean / Rocq / NPA checker を sandbox 実行

Translation:
  translation memory
  glossary
  machine translation draft
  human review workflow

Public API:
  REST / GraphQL
  将来的に RDF / JSON-LD / SPARQL export
```

Wikibase のデータモデルは、扱う情報の概念モデルを明確にし、拡張性・柔軟性・データ交換・JSON/RDFなどでの表現を要件として掲げています。あなたの数学Wikiも、この発想に近い **概念モデル優先** で設計すべきです。([MediaWiki][8])

## 14.2 MediaWikiを使うべきか

選択肢は2つあります。

### A案: MediaWiki + Wikibase + Translate で始める

メリット：

```text
- Wiki編集・履歴・権限・多言語周りが最初から強い
- Wikibase的な構造化データを使える
- Translate拡張で翻訳ワークフローを作りやすい
- 既存コミュニティ運用ノウハウがある
```

デメリット：

```text
- 証明証明書・形式検証・数式検索・依存グラフの深い統合が難しくなりやすい
- UI/UXを完全に数学特化にするには工夫が必要
- 大規模な形式証明ワーカーとの連携は別システムが必要
```

### B案: 独自アプリ + Wikibase風データモデル

メリット：

```text
- 数学エンティティ・証明・形式証明・依存グラフを最初から最適化できる
- UIを定理・証明・翻訳・形式化に特化できる
- AI証明探索やNPA証明器との統合がしやすい
```

デメリット：

```text
- Wiki機能、翻訳、履歴、権限、荒らし対策を自前で作る必要がある
- 初期開発コストが高い
```

私のおすすめは、**最初はB案寄りの設計をしつつ、MediaWiki/Wikibaseの概念を借りる**ことです。つまり、実装は独自でも、データモデルは Wikibase 的にします。

```text
Item = 数学エンティティ
Statement = 数学的関係
Qualifier = 文脈・基礎・条件
Reference = 文献・形式証明・出典
Rank = 推奨定義・標準定理・歴史的表現
```

---

# 15. 編集・レビュー・権限設計

数学Wikiは、Wikipediaよりも強い品質管理が必要です。

## 15.1 ロール

```text
Reader
  閲覧者

Contributor
  下書き投稿・修正提案

Editor
  通常記事編集

Reviewer
  数学的レビュー

Formalizer
  Lean/Rocq/NPA形式化

Translator
  翻訳

Translation Reviewer
  翻訳レビュー

Maintainer
  分野別管理者

Admin
  システム管理
```

## 15.2 レビュー状態

各項目に状態を持たせます。

```text
stub
draft
needs_review
reviewed
needs_formalization
formalized
verified
certified
deprecated
merged
split
```

## 15.3 編集単位

ページ全体ではなく、構造化単位ごとにレビューできるようにします。

```text
定理文レビュー
証明レビュー
翻訳レビュー
形式証明レビュー
文献レビュー
記法レビュー
```

これにより、日本語の翻訳だけ修正したい人、Lean証明だけ追加したい人、文献だけ追加したい人が協力しやすくなります。

---

# 16. バージョン管理

定理や定義は、変更に非常に慎重であるべきです。

## 16.1 Entity revision

すべてのエンティティにrevisionを持たせます。

```json
{
  "entity_id": "T00001234",
  "revision": "rev_00041",
  "statement_hash": "sha256:...",
  "modified_by": "user123",
  "modified_at": "...",
  "change_type": "proof_added"
}
```

## 16.2 主張が変わったら別定理にする

軽微な表現変更なら同じ定理でよいですが、数学的主張が変わる場合は別entityにします。

例：

```text
∀ n : Nat, n + 0 = n
```

から：

```text
∀ n : Int, n + 0 = n
```

に変わるなら別定理です。

同じページ上で「一般化」として接続します。

```text
T00001234 specializes T00004567
```

## 16.3 定義のバージョン

定義が変わると、多くの定理の意味が変わります。

したがって、定義には必ずhashを持たせます。

```text
Definition D000001 version hash
```

定理は、どの定義revisionに依存しているかを記録します。

---

# 17. 公理・基礎論理の扱い

「ありとあらゆる数学」を載せるなら、同じ定理でも基礎が違うことがあります。

```text
ZFC
ZFC + Choice
Constructive type theory
Classical type theory
HoTT / univalent foundations
Setoid-based constructive mathematics
```

したがって、各定理・証明には `foundation_context` を持たせます。

```json
{
  "foundation": "ZFC",
  "uses_choice": true,
  "uses_excluded_middle": true,
  "uses_univalence": false
}
```

形式証明の場合は、axiom report を持たせます。

```json
{
  "axioms_used": [
    "Classical.choice",
    "Propext"
  ]
}
```

これにより、構成的証明と古典的証明を区別できます。

---

# 18. AIの使い方

AIは非常に有用ですが、信頼境界を明確にします。

AIに任せてよいこと：

```text
- 定理ページの下書き生成
- 証明候補生成
- 関連定理推薦
- 翻訳下書き
- 形式化候補生成
- 類似定理の発見
- 文献候補の推薦
```

AIに任せてはいけないこと：

```text
- 未検証証明を verified と表示する
- 定理文を勝手に変更する
- 翻訳をレビュー済みにする
- 形式証明なしに certified と表示する
- 出典を捏造する
```

AI出力には状態を付けます。

```text
ai_draft
needs_human_review
human_reviewed
formal_verified
```

---

# 19. API設計

公開APIは最初から用意した方がよいです。

## 19.1 Entity API

```http
GET /api/entities/T00001234
```

```json
{
  "id": "T00001234",
  "type": "Theorem",
  "labels": {
    "ja": "自然数の加法における右単位元",
    "en": "Right identity of addition on natural numbers"
  },
  "statement": "...",
  "proofs": ["P00000001"],
  "formal_proofs": ["F00000091"]
}
```

## 19.2 Dependency API

```http
GET /api/entities/T00001234/dependencies
```

```json
{
  "direct": ["D00000001", "T00000002"],
  "transitive": ["..."],
  "axioms": []
}
```

## 19.3 Search API

```http
GET /api/search?q=n+%2B+0+%3D+n&lang=ja
```

## 19.4 Formal proof API

```http
GET /api/entities/T00001234/formal-proofs
```

```json
{
  "formal_proofs": [
    {
      "system": "Lean",
      "status": "verified",
      "certificate_hash": "sha256:..."
    }
  ]
}
```

## 19.5 Translation API

```http
GET /api/entities/T00001234/translations
```

```json
{
  "ja": "reviewed",
  "en": "reviewed",
  "fr": "draft",
  "de": "missing"
}
```

---

# 20. ライセンス設計

ライセンスは最初に決めるべきです。

おすすめは：

```text
記事本文:
  CC BY-SA 4.0 または CC BY 4.0

構造化データ:
  CC0 に近い形を検討

形式証明コード:
  Apache-2.0 / MIT / CC0 など、再利用しやすいもの

証明証明書:
  CC0 またはパブリックドメイン相当を検討
```

ただし、既存サイトや本から内容を取り込む場合、ライセンス互換性が非常に重要です。
ProofWiki、nLab、Wikipedia、Stacks Project、Mathlib などの内容をそのまま使う場合は、それぞれのライセンスを確認して、互換性のある形でしか取り込まないようにします。

---

# 21. 初期MVP

いきなり全数学は無理です。
最初は、**設計が将来に耐える小さな範囲**から始めるべきです。

## MVP範囲

```text
分野:
  Logic
  Set / Type basics
  Natural numbers
  Lists
  Elementary algebra
  Groups / monoids
  Basic topology の入口

言語:
  日本語
  英語

機能:
  entity ID
  theorem page
  definition page
  proof page
  translation status
  theorem dependencies
  human review
  simple theorem search
  LaTeX/KaTeX rendering
  formal proof link
```

## MVPで作るページ例

```text
D00000001: 自然数
D00000002: 加法
T00000001: n + 0 = n
T00000002: 0 + n = n
T00000003: 加法の結合律
T00000004: 加法の可換律
D00000010: モノイド
T00000020: モノイドの単位元の一意性
```

この小さな範囲で、次を完成させます。

```text
- 日本語/英語切替
- 定理ID
- 証明ID
- 依存関係
- 翻訳状態
- レビュー状態
- 形式証明へのリンク
- 検索
```

---

# 22. 長期ロードマップ

## Phase A: 基盤

```text
- エンティティモデル
- 多言語ラベル
- 定理・定義・証明ページ
- revision管理
- 基本検索
- Markdown/LaTeX編集
```

## Phase B: 数学知識グラフ

```text
- depends_on
- proves
- uses
- generalizes
- specializes
- equivalent_to
- counterexample_to
- theorem graph visualization
```

## Phase C: 形式証明統合

```text
- Lean/Rocq/NPA formal proof mapping
- certificate hash
- axiom report
- proof status badge
- formal proof search
```

## Phase D: 多言語本格化

```text
- 翻訳メモリ
- 用語辞書
- 翻訳レビュー
- outdated translation detection
- multilingual SEO
```

## Phase E: AI支援

```text
- 証明下書き生成
- 翻訳下書き
- 関連定理推薦
- 形式化候補生成
- 類似定理検索
```

## Phase F: 大規模拡張

```text
- Mathlib mapping
- textbook import
- MSC分類
- advanced formula search
- public API
- dataset export
- RDF/JSON-LD export
```

---

# 23. 最も避けるべき失敗

避けるべきなのはこれです。

```text
- 言語ごとに別々の定理ページを作る
- 定理に安定IDを付けない
- 定理文と証明を同じ非構造化本文に埋め込む
- 定義のバージョンを管理しない
- 証明と定理を1対1に固定する
- 古典公理・選択公理・構成的証明の違いを無視する
- 翻訳を独立記事として放置する
- 形式証明をただの外部リンク扱いにする
- AI生成文をレビュー済みとして扱う
- ページ本文だけを検索対象にして数式・依存関係検索を後回しにする
```

一番危険なのは、最初に普通のブログ/Wiki的なページ構造で始めてしまうことです。
あとから知識グラフ化・多言語同期・形式証明対応を入れるのは非常に難しいです。

---

# 24. 推奨する最終設計

最終的には、こういうシステムを目指すのがよいです。

```text
MathWiki Core:
  数学エンティティDB
  定理・定義・証明・文献・記法・分類

MathWiki Graph:
  依存関係、有向グラフ、一般化、特殊化、使用関係

MathWiki Proof:
  人間向け証明
  形式証明
  証明証明書
  axiom report

MathWiki Translate:
  多言語ラベル
  翻訳単位
  用語辞書
  翻訳メモリ
  レビュー状態

MathWiki Search:
  全文検索
  数式検索
  定理検索
  依存関係検索
  多言語検索

MathWiki AI:
  証明候補
  形式化候補
  翻訳下書き
  関連定理推薦

MathWiki API:
  REST / GraphQL
  JSON-LD / RDF export
  proof certificate export
```

---

# 25. 結論

あなたが作るべきものは、単なる「数学版Wikipedia」ではありません。

より正確には：

```text
多言語対応の数学知識グラフを中心に、
定義・定理・証明・形式証明・文献・依存関係を統合した、
証明付き数学百科プラットフォーム
```

です。

最初から入れるべき中核はこの5つです。

```text
1. 言語非依存の安定ID
2. 定理・定義・証明を分離したエンティティモデル
3. 多言語ラベル・翻訳単位・用語辞書
4. 依存関係グラフ
5. 形式証明・certificate・axiom report への接続
```

短期的には、**日本語・英語対応の小さなMathWiki**として始めればよいです。
ただし内部構造だけは、最初から「全数学・多言語・形式証明・AI探索」に耐える形にしておくべきです。

[1]: https://proofwiki.org/wiki/Main_Page "ProofWiki"
[2]: https://stacks.math.columbia.edu/about "About—The Stacks project"
[3]: https://kerodon.net/tags?utm_source=chatgpt.com "Tags explained"
[4]: https://lean-lang.org/use-cases/mathlib/ "Mathlib: A Foundation for Formal Mathematics Research and Verification — Lean Lang "
[5]: https://www.mediawiki.org/wiki/Wikibase/Reference/en "Wikibase/Reference - MediaWiki"
[6]: https://www.mediawiki.org/wiki/Extension%3ATranslate "Extension:Translate - MediaWiki"
[7]: https://www.wikidata.org/wiki/Help%3AStatements "Help:Statements - Wikidata"
[8]: https://www.mediawiki.org/wiki/Wikibase/DataModel "Wikibase/DataModel - MediaWiki"

