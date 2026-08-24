# API 通信を伴う状態の更新の改善

## 概要

フロントエンドで API 通信を伴う状態更新が、いずれも「通信を待ってから UI を更新する」順序になっている。
そのためAPI通信の待ち時間がそのまま UI に露出している。

加えて `graphql/index.ts` の `defaultOptions` がキャッシュを無効化しており、この待ち時間を増幅させている。

- `watchQuery.fetchPolicy: "network-only"` — 常にネットワーク往復が発生する
- `mutate.refetchQueries: "active"` — mutation ごとにアクティブなクエリを全件再取得する

## 改善方針

UI を先に更新し、その後に API 通信を行う（楽観的更新）。失敗時は変更前の状態へ戻す。
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

- [ ] `onChangeTitle` / `onChangeText` をローカル state + デバウンス送信へ変更する
- [ ] `onInputTitle` / `onInputText` の `useCallback` の依存を `props` 全体から必要な関数のみへ絞る
      （現状は毎レンダーで再生成され `memo` が効いていない）
