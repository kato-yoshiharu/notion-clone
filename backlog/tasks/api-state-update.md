# API 通信を伴う状態の更新の改善

## 概要

フロントエンドで API 通信を伴う状態更新が、いずれも「通信を待ってから UI を更新する」順序になっている。
そのためAPI通信の待ち時間がそのまま UI に露出している。

加えて `graphql/index.ts` の `defaultOptions` がキャッシュを無効化しており、この待ち時間を増幅させている。

- `watchQuery.fetchPolicy: "network-only"` — 常にネットワーク往復が発生する
- `mutate.refetchQueries: "active"` — mutation ごとにアクティブなクエリを全件再取得する

## 改善方針

UI を先に更新し、その後に API 通信を行う（楽観的更新）。失敗時はユーザーへ通知したうえでリロードし、サーバーの状態を取り直す。
タイトルやテキストの入力時のAPI通信時はデバウンスさせる。

## タスク

### 0. デッドコードの削除

`page-list` は props を `_props` として受け取っているが、使用していないので、削除する。
それを渡すために上流に置かれているものがまとめて未使用になっているので、それも削除する。

- [x] `PageListProps` と、`Sidebar` / `page-page` の対応する props を削除する
- [x] `page-page` の `useListRootPagesQuery` / `pageListResult` と、
      `page-page` 側の `onClickAddPage` / `onClickRemovePageButton` を削除する

### 1. 高頻度な更新のデバウンス

`page-content` は `onInput`（＝1 キーストロークごと）に `updatePage` を発火し、しかも `await` している。
デバウンスが無いため入力のたびに mutation が飛び、
さらに `refetchQueries: "active"` によってアクティブなクエリが全件再取得される。本タスク中で最も影響が大きい。

`title` / `text` はローカル state を持たずサーバー値を直接描画しているため、
`useEffect` 側に「編集中はキャレットが戻るので書き換えない」という回避が入っている。

- [x] `onChangeTitle` / `onChangeText` をデバウンス送信へ変更する（未送信分は blur とアンマウントで flush する）
- [x] `onInputTitle` / `onInputText` の `useCallback` の依存を `props` 全体から必要な関数のみへ絞る
      （現状は毎レンダーで再生成され `memo` が効いていない）

### 2. `page-list` の楽観的更新

`page-list` の各ハンドラが `await` 後に `setTree` している。
サーバーの戻り値に依存していないものは、順序を入れ替えるだけで楽観的更新にできる。

- [x] `onClickRename`
- [x] `onClickDelete`
- [x] `onMove`
- [x] `onClickCollapse`（一度取得した `children` を保持しているのに開くたび
      `listChildrenPages` を叩き直しているため、既存の `children` があれば即座に開き再取得は背後で行う）

#### 関数型更新への移行

各ハンドラはレンダー時点の `tree` を元に `setTree` しており、`await` を挟むと古い `tree` で上書きしてしまう可能性がある。
関数型更新へ移す。

- [x] `setPageTree` の型を `Dispatch<SetStateAction<PageTree>>` へ変更する
- [x] 各ハンドラを `setPageTree(prev => next)` の形へ移行する

#### 失敗時の扱い

現在の `invariant` は `useCallback` の async 関数の中で投げているため React に捕捉されず unhandled rejection として消えており、
UI を先に更新すると「画面上は反映されたがサーバーには反映されていない」状態が黙って残る。

- [x] 失敗時の扱いと、失敗をユーザーへ通知する経路を共通ヘルパーとして 1 箇所に置く

巻き戻し方は「逆適用」と「スナップショット復元」の 2 通りを検討したが、少し複雑なため、以下の対応にした。
失敗はレアケースのため、共通ヘルパー `useOptimisticPageTree` は巻き戻しを持たず、
アラートを表示したうえでリロードし、サーバーの状態を取り直すことにした。

#### `EnsurePageExists` の暴発

- [x] `EnsurePageExists` を middleware から `useEnsurePageExists` へ移し、`onClickDelete` から呼ぶ

`EnsurePageExists` は「木が非空から空になった」遷移を検知して `addPage` + `router.push` していた。
楽観的更新では削除の成否が出る前に木が空になるため、通信の完了を待たずに暴発する。
監視を残すなら通信中は effect を抑止するフラグが要るが、
そもそもこの middleware が発火するのは「最後の 1 ページを削除したとき」だけなので、
**木の監視をやめ、`onClickDelete` の `commit` が成功した後で補充を呼ぶ**形にした。

### 3. ページ追加の楽観的更新

`page-list` の `onClickAddRoot` / `onClickAddChild` と `useEnsurePageExists` は、
新規ノードの `id` が `addPage` のレスポンス由来のため、順序を入れ替えるだけでは成立しない。
先に描画するには id を先に確定させる必要がある。

- 案 A: バックエンドが id を受け取れるようにし、フロントで採番する（採用）
- 案 B: 仮 id で描画し、レスポンス到着後に本物の id へ差し替える
  - 差し替え完了までの間にそのノードを操作されると仮 id のまま mutation が飛ぶため、操作の抑止が必要

案 B は「差し替え待ちのノード」という状態を全ハンドラが意識する必要があり、
タスク 2 の共通ヘルパーとも噛み合わない。案 A を採る。

案 A のコストは小さい。`notion.pages.id` は既に `UUID DEFAULT uuid_generate_v4()` のため
**マイグレーションは不要**で、id が渡されたときだけ `DEFAULT` の代わりに使う形にすればよい。

- [x] `models::page::AddPage` に `id: Option<PageId>` を追加する
- [x] `apis/page.rs` の `AddPage` インプットに `id` を追加し、`From` 実装で引き回す
- [x] `repositories/postgres/page/mod.rs` の `add` の INSERT を、
      `id` が `Some` のときは明示指定・`None` のときは従来どおり採番する形にする
- [x] フロントで `crypto.randomUUID()` により採番し、`onClickAddRoot` / `onClickAddChild` /
      `useEnsurePageExists` をタスク 2 の共通ヘルパー経由の楽観的更新へ移行する

INSERT は `VALUES (COALESCE($1, uuid_generate_v4()), ...)` とした。
一文のままで済み、`id` が `None` のときの採番はカラムの `DEFAULT` と同じ式になる。

なお `useEnsurePageExists` は `onClickDelete` の `commit` の中から呼ばれるため、
共通ヘルパーを二重に通さないよう、そこだけは `setPageTree` と `addPage` を直に並べている。
