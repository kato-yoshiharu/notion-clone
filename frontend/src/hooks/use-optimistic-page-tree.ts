import { useCallback } from "react";

import { PageTree, usePageTree } from "@/global-states/page-tree";

interface OptimisticUpdate {
  /** 先にUIへ反映する更新 */
  apply: (prev: PageTree) => PageTree;
  /** APIへの送信。失敗時は例外を投げる */
  commit: () => Promise<void>;
  /** 失敗時にユーザーへ見せる文言 */
  errorMessage: string;
}

/**
 * 楽観的更新のヘルパー。
 * UIを先に更新し、失敗したらユーザーに通知したうえでリロードしてサーバーの状態に揃える。
 */
export const useOptimisticPageTree = () => {
  const { setPageTree } = usePageTree();

  return useCallback(
    async ({ apply, commit, errorMessage }: OptimisticUpdate) => {
      setPageTree(apply);

      try {
        await commit();
      } catch {
        // 巻き戻しは行わず、リロードでサーバーの状態を取り直す
        // TODO: トーストなどに差し替える
        window.alert(errorMessage);
        window.location.reload();
      }
    },
    [setPageTree],
  );
};
