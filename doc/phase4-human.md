以下は **Phase 4: tactic** の詳細設計です。
Phase 3 までで、表層構文を core term に落とせるようになりました。Phase 4 では、`_` や `?m` で生じた未解決 goal を、人間やAIが小さな命令で解けるようにします。

対象 tactic はこの6つです。

```text
intro
exact
apply
rw
simp-lite
induction
```

大原則は変わりません。

```text
tactic は信用しない。
tactic は proof term を組み立てるだけ。
最後に kernel が proof term を検査する。
```

この Phase 4 文書内の tactic script 例では、読みやすさのため `0` を使うことがあります。
Phase 3 MVP の実入力では、数値リテラルを入れるまでは `Nat.zero` か開いた namespace 内の
`zero` と書ければ十分です。tactic が組み立てる core proof term には `Nat.zero` への
canonical `Const` 参照だけが残ります。

---

# 1. tactic 層の基本モデル

tactic は、証明状態を変換する関数です。

```text
ProofState → TacticResult
```

もう少し正確には：

```text
tactic:
  現在の goal を読み、
  その goal を満たす proof term の一部を作り、
  必要なら新しい subgoal を生成する。
```

たとえば：

```npa
theorem t : A → A := by
  intro x
  exact x
```

は内部的にはこうなります。

```text
初期goal:
  ⊢ A → A

intro x:
  x : A
  ⊢ A

exact x:
  closed
```

しかし kernel に渡る最終証明は tactic script ではありません。

```text
λ x : A, x
```

という core proof term です。

---

# 2. ProofState の構造

Phase 4 では、hole / metavariable を goal として扱います。

```rust
struct ProofState {
    goals: Vec<GoalId>,
    metavars: MetaVarStore,
    env: Env,
}
```

各 goal は：

```rust
struct Goal {
    id: GoalId,
    context: LocalContext,
    target: ExprId,
    assignment: Option<ExprId>,
    source_span: Option<Span>,
}
```

表示例：

```text
goal g1

n : Nat
⊢ n = n
```

内部的には：

```text
?g1 : Eq Nat n n
```

という metavariable です。

tactic が goal を解くとは、基本的には：

```text
?g1 := proof_term
```

と代入することです。

新しい subgoal が出る場合は、proof term の中に新しい metavariable を埋めます。

---

# 3. tactic の共通 API

```rust
trait Tactic {
    fn run(&self, state: &mut ProofState, goal: GoalId) -> Result<TacticOutcome>;
}
```

結果：

```rust
struct TacticOutcome {
    solved_goal: GoalId,
    new_goals: Vec<GoalId>,
    proof_delta: ProofDelta,
    messages: Vec<Message>,
}
```

外部 API なら：

```json
{
  "state_id": "s1",
  "goal_id": "g1",
  "tactic": "intro n"
}
```

結果：

```json
{
  "status": "success",
  "new_state_id": "s2",
  "closed_goals": [],
  "new_goals": [
    {
      "goal_id": "g2",
      "context": [
        {"name": "n", "type": "Nat"}
      ],
      "target": "n = n"
    }
  ]
}
```

失敗時：

```json
{
  "status": "error",
  "error_kind": "expected_pi_type",
  "message": "`intro` can only be used when the target is a function type or forall."
}
```

---

# 4. `intro`

## 4.1 役割

`intro` は、goal の target が `Pi` 型、つまり関数型や全称命題のとき、仮定を文脈に追加します。

例：

```npa
theorem id_nat : Nat → Nat := by
  intro n
  exact n
```

初期状態：

```text
⊢ Nat → Nat
```

`intro n` 後：

```text
n : Nat
⊢ Nat
```

## 4.2 core proof term

`intro` は lambda を作ります。

```text
goal:
  ⊢ Π x : A, B

intro x:
  new goal:
    x : A ⊢ B

old goal assignment:
  ?g := λ x : A, ?g_new
```

つまり：

```text
?g : Π x : A, B
?g := Lam x : A, ?g_new
```

## 4.3 アルゴリズム

```rust
fn tactic_intro(state: &mut ProofState, goal_id: GoalId, name: NameId) -> Result<()> {
    let goal = state.get_goal(goal_id);
    let target = whnf(state.env, &goal.context, goal.target)?;

    match target {
        ExprKind::Pi { ty: domain, body, binder_info, .. } => {
            let local = state.context_add_local(goal.context, name, domain);
            let new_target = instantiate_with_local(body, local);
            let new_goal = state.new_goal(local.context, new_target);

            let proof = mk_lam(name, domain, mk_mvar(new_goal));
            state.assign_goal(goal_id, proof);
            state.replace_goal(goal_id, new_goal);

            Ok(())
        }
        _ => Err(Error::ExpectedPiType),
    }
}
```

## 4.4 対応する対象

`intro` が使えるもの：

```text
⊢ A → B
⊢ ∀ x : A, P x
⊢ Π x : A, B x
```

使えないもの：

```text
⊢ Nat
⊢ n = n
⊢ A ∧ B
```

`A ∧ B` に対しては、将来的には `constructor` を使います。Phase 4 MVPでは `constructor` はまだ入れなくてもよいです。

---

# 5. `exact`

## 5.1 役割

`exact t` は、現在の goal を term `t` で直接閉じます。

例：

```npa
theorem self_eq (n : Nat) : n = n := by
  exact Eq.refl n
```

goal：

```text
n : Nat
⊢ n = n
```

`exact Eq.refl n` は、`Eq.refl n` が target と同じ型を持つことを確認し、goal を閉じます。

## 5.2 core proof term

```text
?g := elaborated_term
```

だけです。

## 5.3 アルゴリズム

```rust
fn tactic_exact(state: &mut ProofState, goal_id: GoalId, term: SurfaceExpr) -> Result<()> {
    let goal = state.get_goal(goal_id);

    let core = elaborate_check(
        &state.env,
        &goal.context,
        term,
        goal.target,
    )?;

    if has_unsolved_metas(core) {
        return Err(Error::UnsolvedMetasInExact);
    }

    state.assign_goal(goal_id, core);
    state.remove_goal(goal_id);

    Ok(())
}
```

## 5.4 `exact` の設計判断

Phase 4 MVPでは、`exact` は conservative にします。

```text
exact t
```

で `t` の中に未解決 metavariable が残った場合は失敗させます。

つまり：

```npa
exact _
```

は `exact` としては失敗。

ただし将来的には、`refine` を導入して：

```npa
refine f ?_
```

のように subgoal を明示的に残せるようにします。

Phase 4 では：

```text
exact:
  完全に閉じる

apply:
  必要なsubgoalを生成してよい
```

という分担が分かりやすいです。

---

# 6. `apply`

## 6.1 役割

`apply f` は、現在の target を、ある定理・仮定・関数 `f` の結論に一致させ、必要な前提を subgoal にします。

例：

```npa
theorem trans_example
  (A : Type)
  (x y z : A)
  (h1 : x = y)
  (h2 : y = z)
  : x = z := by
  apply Eq.trans
  exact h1
  exact h2
```

概念的には：

```text
Eq.trans : x = y → y = z → x = z
```

なので、goal `x = z` に `apply Eq.trans` すると：

```text
subgoal 1:
  ⊢ x = ?y

subgoal 2:
  ⊢ ?y = z
```

のようになります。`?y` は unification で決まることもあります。

## 6.2 core proof term

`apply f` は次の形を作ります。

```text
?g := f ?a1 ?a2 ... ?an
```

そして未解決の `?ai` が新しい goal になります。

## 6.3 基本アルゴリズム

```text
goal target:
  T

term f:
  f : Π x₁ : A₁, Π x₂ : A₂, ..., R

apply f:
  R を T に unify する
  各 xᵢ に対して必要なら metavariable ?mᵢ を作る
  ?g := f ?m₁ ?m₂ ...
  未解決 ?mᵢ を subgoal にする
```

疑似コード：

```rust
fn tactic_apply(state: &mut ProofState, goal_id: GoalId, f_expr: SurfaceExpr) -> Result<()> {
    let goal = state.get_goal(goal_id);

    let (mut f_core, mut f_ty) = elaborate_infer(
        &state.env,
        &goal.context,
        f_expr,
    )?;

    let mut args = Vec::new();
    let mut new_goals = Vec::new();

    loop {
        let ty_whnf = whnf(state.env, &goal.context, f_ty)?;

        match ty_whnf {
            ExprKind::Pi { ty: domain, body, binder_info, .. } => {
                let m = state.new_meta(goal.context.clone(), domain);

                args.push(m.as_expr());
                f_core = mk_app(f_core, m.as_expr());
                f_ty = instantiate(body, m.as_expr());

                if binder_info.is_explicit_or_proof_relevant() {
                    new_goals.push(m.goal_id());
                }

                // 試しに結論が target と unify できるか確認する
                if can_unify(state, f_ty, goal.target) {
                    unify(state, f_ty, goal.target)?;
                    break;
                }
            }
            _ => {
                unify(state, ty_whnf, goal.target)?;
                break;
            }
        }
    }

    state.assign_goal(goal_id, f_core);
    state.replace_goal_with_many(goal_id, new_goals);

    Ok(())
}
```

## 6.4 `apply` で重要な点

`apply` では、結論が target に合うまで引数を補っていきます。

例：

```text
f : A → B → C
goal:
  ⊢ C
```

なら：

```text
?g := f ?a ?b
```

新しい goal：

```text
⊢ A
⊢ B
```

です。

ただし、implicit arguments は goal にしない方がよいです。
それらは unification で解けることが多いからです。

```text
implicit metavariable:
  elaborator/unifier が解く

explicit/proof argument:
  subgoal としてユーザーに出す
```

## 6.5 失敗条件

```text
- f が関数型でない
- f の結論が target と unify できない
- implicit metavariable が解けず、許容できない
- occurs check に失敗する
```

エラー例：

```text
cannot apply `Nat.succ`

target:
  n = n

`Nat.succ` has type:
  Nat → Nat

which does not match the target.
```

---

# 7. `rw`

## 7.1 役割

`rw [h]` は、等式 `h` を使って target または仮定を書き換えます。

例：

```npa
theorem rw_example
  (a b : Nat)
  (h : a = b)
  : a = a := by
  rw [h]
```

target：

```text
a = a
```

`rw [h]` 後：

```text
b = b
```

または occurrence 指定によって片方だけ書き換えることもできます。Phase 4 MVPでは、まず「target内の全出現」を書き換えるだけでよいです。

## 7.2 rewrite rule の形

`rw` に渡すものは、基本的には等式証明です。

```text
h : Eq A lhs rhs
```

これを使って：

```text
lhs  ↦ rhs
```

と書き換えます。

逆向きは：

```npa
rw [← h]
```

または：

```npa
rw [<- h]
```

とします。

## 7.3 何を書き換えるか

Phase 4 MVPでは、targetだけを書き換えます。

```npa
rw [h]
```

は target を書き換える。

仮定を書き換える構文：

```npa
rw [h] at h2
rw [h] at *
```

は後で追加します。

## 7.4 `rw` の内部処理

```text
1. h を elaboration する
2. h の型を WHNF する
3. Eq A lhs rhs の形か確認する
4. target の中で lhs に一致する部分を探す
5. その部分を rhs に置換した new_target を作る
6. 古い target の証明から新しい target の証明を作る変換を生成する
7. 新しい goal を作る
```

重要なのは、target を変えるだけではなく、**proof term の変換**も必要なことです。

## 7.5 proof term の考え方

target が：

```text
P lhs
```

で、rewrite により：

```text
P rhs
```

になったとします。

等式：

```text
h : lhs = rhs
```

を使えば、`P rhs` の証明から `P lhs` の証明を作れます。

なぜなら、元の goal は `P lhs` だからです。

つまり `rw [h]` は普通、ユーザーには新しい goal `P rhs` を見せますが、内部では：

```text
?old_goal := Eq.subst h ?new_goal
```

のような proof term を作ります。

向きには注意が必要です。

```text
old target:
  P lhs

new target:
  P rhs

new goal proof:
  ?new : P rhs

old proof:
  Eq.subst h ?new : P lhs
```

実際の `Eq.subst` の型や向きは設計次第なので、実装時には正確に合わせます。

## 7.6 MVPでの簡略化

`rw` は本気で作ると難しいです。Phase 4ではかなり制限して始めるのがよいです。

Phase 4 MVP の `rw`：

```text
- Eq のみ対応
- target のみ対応
- 左から右、右から左を明示
- 最初に見つかった全出現または最初の1出現だけ
- dependent rewrite は限定的
- rewrite proof は Eq.rec / Eq.subst で生成
```

まだ入れないもの：

```text
- setoid rewrite
- rewriting under binders
- rewriting in hypotheses
- occurrence selection
- simp attributesとの統合
- heterogeneous equality
- conditional rewrite
```

## 7.7 rewrite 検索

target 内で lhs に一致する subterm を探します。

```text
target:
  Eq Nat (Nat.add n Nat.zero) n

rule:
  Nat.add ?x Nat.zero ↦ ?x
```

一致：

```text
?x := n
```

置換後：

```text
Eq Nat n n
```

これで `Eq.refl n` が使えるようになります。

---

# 8. `simp-lite`

## 8.1 役割

`simp-lite` は、登録済みの単純な rewrite rule を使って target を自動簡約する tactic です。

Lean の `simp` の本格版は非常に強力ですが、最初から同じものを作るのは難しいです。
Phase 4では、**小さいが proof-producing な simplifier** を作ります。

例：

```npa
theorem add_zero_example (n : Nat) : n + 0 = n := by
  simp-lite
```

`simp-lite` は `Nat.add_zero` や定義展開を使って target を：

```text
n = n
```

に簡約し、`Eq.refl n` で閉じます。

## 8.2 simp rule

simp rule は基本的に等式定理です。

```text
Nat.add_zero : ∀ n : Nat, n + 0 = n
Nat.zero_add : ∀ n : Nat, 0 + n = n
```

内部では rewrite rule として登録します。

```rust
struct SimpRule {
    theorem: GlobalRef,
    lhs: ExprId,
    rhs: ExprId,
    orientation: RewriteDirection,
    priority: u32,
}
```

## 8.3 simp-lite の処理

```text
1. target を WHNF する
2. target 内の各 subterm に rewrite rule を試す
3. 成功したら置換する
4. これを停止するまで繰り返す
5. 最終 target が reflexive equality なら Eq.refl で閉じる
6. 閉じられなければ、新しい簡約後 goal を残す
```

Phase 4では、閉じられない場合に失敗させてもよいです。

```text
simp-lite:
  target を完全に閉じられたら成功
  閉じられなければ失敗
```

後で：

```text
simp:
  targetを簡約して新しいgoalを残す
```

という挙動も追加できます。

## 8.4 proof-producing であること

`simp-lite` は単に target を書き換えるだけでは不十分です。

各 rewrite について、proof term を記録します。

```text
target0
  -- rewrite by Nat.add_zero
target1
  -- rewrite by Nat.zero_add
target2
  -- refl
closed
```

最終的に：

```text
?g := proof_composed_from_rewrites
```

を作ります。

簡略化のため、Phase 4では `simp-lite` の出力をこうしてもよいです。

```text
1. target を簡約した target'
2. target' が Eq t t なら Eq.refl t
3. target と target' の等価性証明を Eq.subst チェーンで作る
```

## 8.5 停止性

`simp-lite` は無限ループを避ける必要があります。

危険な rule：

```text
x = x + 0
```

これを左から右に使うと式が大きくなります。

Phase 4 MVPでは、simp rule 登録時に制限します。

```text
- lhs のサイズ > rhs のサイズ なら許可
- または明示的に safe とマークされた rule のみ許可
- 同じ target hash を再訪したら停止
- 最大 rewrite 回数を設ける
```

## 8.6 simp-lite に入れる最小 rule

最初はこれだけで十分です。

```text
Nat.add_zero
Nat.zero_add
Nat.add_succ
Nat.succ_add
Nat.mul_zero
Nat.zero_mul
Eq.refl 関連
βζδ reduction
```

ただし、`Nat.add` を第2引数再帰で定義しているなら：

```text
n + 0
```

は βδι reduction だけで `n` になるため、`Nat.add_zero` すら不要な場合があります。

---

# 9. `induction`

## 9.1 役割

`induction n` は、帰納型の値 `n` について帰納法を行います。

例：

```npa
theorem zero_add (n : Nat) : 0 + n = n := by
  induction n
  case zero =>
    exact Eq.refl 0
  case succ n ih =>
    simp-lite
```

初期 goal：

```text
n : Nat
⊢ 0 + n = n
```

`induction n` 後：

```text
case zero:
  ⊢ 0 + 0 = 0

case succ:
  n : Nat
  ih : 0 + n = n
  ⊢ 0 + succ n = succ n
```

## 9.2 core proof term

`induction n` は、内部的には `Nat.rec` を使います。

target が `P n` の形なら：

```text
Nat.rec
  motive
  base_case
  step_case
  n
```

です。

ここで：

```text
motive := λ n : Nat, target_with_n
```

例：

```text
target:
  Eq Nat (Nat.add Nat.zero n) n
```

なら：

```text
motive :=
  λ n : Nat, Eq Nat (Nat.add Nat.zero n) n
```

base case：

```text
motive Nat.zero
```

step case：

```text
Π n : Nat, motive n → motive (Nat.succ n)
```

## 9.3 アルゴリズム

```text
goal:
  context Γ, n : Nat
  target T

induction n:
  1. target T から motive P := λ n, T を作る
  2. base goal を作る:
       Γ ⊢ P Nat.zero
  3. step goal を作る:
       Γ, n : Nat, ih : P n ⊢ P (Nat.succ n)
  4. old goal に Nat.rec P ?base ?step n を代入
```

疑似コード：

```rust
fn tactic_induction_nat(
    state: &mut ProofState,
    goal_id: GoalId,
    var_name: NameId,
) -> Result<()> {
    let goal = state.get_goal(goal_id);
    let local = goal.context.lookup(var_name)?;

    ensure_type_is_nat(local.ty)?;

    let motive = abstract_over_local(goal.target, local);
    // motive : Nat -> Sort u

    let base_target = mk_app(motive, Nat.zero);
    let base_goal = state.new_goal(goal.context.without_or_with(local), base_target);

    let n_local = fresh_local("n", Nat);
    let ih_ty = mk_app(motive, n_local.expr);
    let ih_local = fresh_local("ih", ih_ty);

    let step_target = mk_app(motive, mk_app(Nat.succ, n_local.expr));
    let step_ctx = goal.context
        .replace_or_extend(local, n_local)
        .add(ih_local);

    let step_goal = state.new_goal(step_ctx, step_target);

    let proof = mk_nat_rec(
        motive,
        mk_mvar(base_goal),
        mk_lam(n_local, mk_lam(ih_local, mk_mvar(step_goal))),
        local.expr,
    );

    state.assign_goal(goal_id, proof);
    state.replace_goal_with_many(goal_id, vec![base_goal, step_goal]);

    Ok(())
}
```

## 9.4 重要な問題: target が変数に依存しているか

`induction n` では、target の中の `n` を抽象化して motive を作ります。

```text
target:
  0 + n = n

motive:
  λ k : Nat, 0 + k = k
```

この抽象化が正しくできないと、帰納法が壊れます。

そのため、Phase 4 MVPでは、まず `Nat` に限定し、対象変数が local context に直接ある場合だけ対応するのがよいです。

対応する：

```text
n : Nat
⊢ P n
```

まだ対応しない：

```text
f n : Nat
⊢ P (f n)
```

複雑な dependent induction は後回しにします。

## 9.5 context 中の仮定の扱い

たとえば：

```text
n : Nat
h : Q n
⊢ P n
```

で `induction n` すると、`h` も `n` に依存します。

本格的には dependent context abstraction が必要です。

Phase 4 MVPでは簡略化します。

```text
制限:
  induction対象 n に依存する後続仮定がある場合は失敗
```

エラー：

```text
cannot perform simple induction on `n`
because hypothesis `h` depends on `n`.

Hint:
  generalize or revert dependent hypotheses first.
```

将来的には `revert`, `generalize`, dependent induction を追加します。

---

# 10. tactic parser

Phase 4 では、簡単な tactic script parser も必要です。

```npa
by
  intro n
  exact n
```

最小文法：

```text
tactic_script ::=
  "by" tactic_seq

tactic_seq ::=
  tactic*

tactic ::=
    "intro" ident
  | "exact" term
  | "apply" term
  | "rw" "[" rw_rule_list "]"
  | "simp-lite"
  | "induction" ident
```

`rw` の例：

```text
rw [h]
rw [<- h]
rw [Nat.add_zero]
```

Phase 4では case syntax は簡略化してよいです。

```npa
induction n
exact Eq.refl 0
simp-lite
```

ただし、複数 goal がある場合、tactic は常に先頭 goal に適用する、というルールにします。

```text
current goals = [g1, g2, g3]
tactic applies to g1
```

将来的には：

```npa
case zero =>
  ...
case succ n ih =>
  ...
```

を導入します。

---

# 11. tactic 実行の全体フロー

```text
1. theorem の右辺が `by ...` なら proof mode に入る
2. theorem type から初期 goal ?g を作る
3. tactic を順に実行する
4. 各 tactic が goal を代入し、新しい goal を作る
5. 最後に goal が0個なら proof term を取り出す
6. kernel で proof : theorem_type を検査する
7. certificate に保存する
```

疑似コード：

```rust
fn elaborate_by_proof(theorem_type: ExprId, tactics: Vec<TacticSyntax>) -> Result<ExprId> {
    let mut state = ProofState::new(theorem_type);

    for tac in tactics {
        let goal = state.current_goal()
            .ok_or(Error::NoGoalsButTacticRemaining)?;

        run_tactic(&mut state, goal, tac)?;
    }

    if !state.goals.is_empty() {
        return Err(Error::UnsolvedGoals(state.goals));
    }

    let proof = state.extract_root_proof()?;

    kernel_check(proof, theorem_type)?;

    Ok(proof)
}
```

---

# 12. 6つの tactic の関係

| tactic      | 何をするか                  | proof term                     |
| ----------- | ---------------------- | ------------------------------ |
| `intro`     | `Π` target を分解し、仮定を追加  | `λ x, ?g`                      |
| `exact`     | term で goal を直接閉じる     | `t`                            |
| `apply`     | 定理・仮定を使い、前提を subgoal 化 | `f ?a ?b ...`                  |
| `rw`        | 等式で target を書き換える      | `Eq.subst` / `Eq.rec`          |
| `simp-lite` | 登録済み rewrite で簡約して閉じる  | rewrite proof chain            |
| `induction` | 帰納法で case 分割           | `Nat.rec motive ?base ?step n` |

---

# 13. Phase 4 の最小テスト

## 13.1 `intro` + `exact`

```npa
theorem id_nat : Nat → Nat := by
  intro n
  exact n
```

期待される core proof：

```text
λ n : Nat, n
```

## 13.2 `Eq.refl`

```npa
theorem self_eq (n : Nat) : n = n := by
  exact Eq.refl n
```

確認：

```text
implicit arg Nat が補完される
exact が goal を閉じる
```

## 13.3 `apply`

```npa
theorem use_id (n : Nat) : Nat := by
  apply id
  exact Nat
  exact n
```

ただし `id` の implicit 引数を使えるなら：

```npa
theorem use_id (n : Nat) : Nat := by
  apply id
  exact n
```

## 13.4 `rw`

```npa
theorem rw_test (a b : Nat) (h : a = b) : a = a := by
  rw [h]
  exact Eq.refl b
```

実装方針によっては `rw [h]` 後の target は：

```text
b = b
```

になります。

## 13.5 `simp-lite`

```npa
theorem add_zero (n : Nat) : n + 0 = n := by
  simp-lite
```

期待：

```text
n + 0 が n に簡約される
target が n = n になる
Eq.refl n で閉じる
```

## 13.6 `induction`

```npa
theorem zero_add (n : Nat) : 0 + n = n := by
  induction n
  exact Eq.refl 0
  simp-lite
```

期待：

```text
base:
  0 + 0 = 0

step:
  n : Nat
  ih : 0 + n = n
  ⊢ 0 + succ n = succ n
```

---

# 14. Phase 4 でまだ入れないもの

Phase 4 MVPでは、以下は後回しにします。

```text
- constructor
- cases
- refine
- have
- specialize
- assumption
- contradiction
- calc
- case syntax
- rewrite in hypotheses
- occurrence selection
- full dependent induction
- typeclass-driven apply
- full simp
- ring / omega / linarith
```

特に `rw` と `induction` は見た目より難しいです。
最初は **Nat と Eq に限定した安全な最小版** を作るのがよいです。

---

# 15. Phase 4 の完了条件

Phase 4 が完了したと言える条件はこれです。

```text
- `by` proof block を parse できる
- ProofState / Goal / MetaVar が実装されている
- tactic が現在 goal に適用できる
- `intro` が Pi target を lambda に変換できる
- `exact` が term で goal を閉じられる
- `apply` が theorem/仮定を使って subgoal を作れる
- `rw` が Eq による target rewrite を行える
- `simp-lite` が単純rewriteとreflで goal を閉じられる
- `induction` が Nat.rec による base/step goal を作れる
- tactic 後の proof term を kernel が検査できる
- unresolved goal が残った場合は certificate 化を拒否できる
```

---

一文でまとめると、Phase 4 は **「人間やAIが扱いやすい小さな証明命令を、kernel が検査できる core proof term に変換する層」** です。

最初に実装すべき順番は：

```text
1. exact
2. intro
3. apply
4. rw
5. simp-lite
6. induction
```

です。`exact` と `intro` で proof state の基本が固まり、`apply` で subgoal 生成が入り、`rw` と `simp-lite` で等式推論が使えるようになり、最後に `induction` で帰納法に到達します。
