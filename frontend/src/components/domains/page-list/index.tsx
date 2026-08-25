import { invariant } from "@suimenkathemove/utils";
import { memo, useCallback, useEffect } from "react";
import {
  Tree,
  moveNode,
  removeNode,
  updateNode,
} from "react-notion-sortable-tree";

import {
  SortableTree,
  SortableTreeProps,
} from "@/components/domains/sortable-tree";
import { PageTree, usePageTree } from "@/global-states/page-tree";
import {
  MoveTarget,
  MoveTargetType,
  PageId,
  useAddPageMutation,
  useListChildrenPagesLazyQuery,
  useListRootPagesLazyQuery,
  useMovePageMutation,
  useRemovePageMutation,
  useUpdatePageMutation,
} from "@/graphql/generated";
import { useEnsurePageExists } from "@/hooks/use-ensure-page-exists";
import { useOptimisticPageTree } from "@/hooks/use-optimistic-page-tree";

type Data = {
  title: string;
};

export const PageList = memo(() => {
  const { pageTree: tree, setPageTree: setTree } = usePageTree();

  const optimistic = useOptimisticPageTree();
  const ensurePageExists = useEnsurePageExists();

  const [listRootPages] = useListRootPagesLazyQuery();
  const [listChildrenPages] = useListChildrenPagesLazyQuery();
  const [addPage] = useAddPageMutation();
  const [updatePage] = useUpdatePageMutation();
  const [removePage] = useRemovePageMutation();
  const [movePage] = useMovePageMutation();

  useEffect(() => {
    void (async () => {
      const result = await listRootPages();
      invariant(
        result.data?.listRootPages.__typename === "ListPages",
        "TODO: error handling",
      );
      const newTree: Tree<Data> = result.data.listRootPages.items.map((r) => ({
        id: r.id,
        children: [],
        collapsed: true,
        data: {
          title: r.title,
        },
      }));
      setTree(newTree);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onClickCollapse: SortableTreeProps["onClickCollapse"] = useCallback(
    async (item) => {
      // 折りたたみはAPI通信を伴わないローカル更新
      if (!item.collapsed) {
        setTree((prev) =>
          updateNode(prev, item.id, (node) => ({
            ...node,
            collapsed: true,
          })),
        );

        return;
      }

      // 既存の`children`があれば即座に開き、再取得は背後で行う
      await optimistic({
        apply: (prev) =>
          updateNode(prev, item.id, (node) => ({
            ...node,
            collapsed: false,
          })),
        commit: async () => {
          const result = await listChildrenPages({
            variables: { id: item.id as PageId },
          });
          invariant(
            result.data?.listChildrenPages.__typename === "ListPages",
            "TODO: error handling",
          );
          const children = result.data.listChildrenPages.items;
          setTree((prev) =>
            updateNode(prev, item.id, (node) => ({
              ...node,
              children: children.map((c) => ({
                id: c.id,
                children: [],
                collapsed: true,
                data: {
                  title: c.title,
                },
              })),
            })),
          );
        },
        errorMessage: "ページの取得に失敗しました",
      });
    },
    [listChildrenPages, optimistic, setTree],
  );

  const onClickAddRoot: SortableTreeProps["onClickAddRoot"] =
    useCallback(async () => {
      const id = crypto.randomUUID() as PageId;

      await optimistic({
        apply: (prev) =>
          prev.concat({
            id,
            children: [],
            collapsed: true,
            data: {
              title: "",
            },
          }),
        commit: async () => {
          const result = await addPage({
            variables: {
              parentId: null,
              addPage: { id, title: "", text: "" },
            },
          });
          invariant(
            result.data?.addPage.__typename === "Page",
            "TODO: error handling",
          );
        },
        errorMessage: "ページの追加に失敗しました",
      });
    }, [addPage, optimistic]);

  const onClickAddChild: SortableTreeProps["onClickAddChild"] = useCallback(
    async (parentId) => {
      const id = crypto.randomUUID() as PageId;

      await optimistic({
        apply: (prev) =>
          updateNode(prev, parentId, (node) => ({
            ...node,
            children: node.children.concat({
              id,
              children: [],
              collapsed: true,
              data: {
                title: "",
              },
            }),
            collapsed: false,
          })),
        commit: async () => {
          const result = await addPage({
            variables: {
              parentId: parentId as PageId,
              addPage: { id, title: "", text: "" },
            },
          });
          invariant(
            result.data?.addPage.__typename === "Page",
            "TODO: error handling",
          );
        },
        errorMessage: "ページの追加に失敗しました",
      });
    },
    [addPage, optimistic],
  );

  const onClickRename: SortableTreeProps["onClickRename"] = useCallback(
    async (item) => {
      const value = window.prompt("", item.data.title) ?? "";

      await optimistic({
        apply: (prev) =>
          updateNode(prev, item.id, (node) => ({
            ...node,
            data: {
              title: value,
            },
          })),
        commit: async () => {
          const result = await updatePage({
            variables: { id: item.id as PageId, updatePage: { title: value } },
          });
          invariant(
            result.data?.updatePage.__typename === "Page",
            "TODO: error handling",
          );
        },
        errorMessage: "ページ名の変更に失敗しました",
      });
    },
    [optimistic, updatePage],
  );

  const onClickDelete: SortableTreeProps["onClickDelete"] = useCallback(
    async (id) => {
      let removedTree: PageTree = [];

      await optimistic({
        apply: (prev) => {
          const [newTree] = removeNode(prev, id);
          removedTree = newTree;

          return newTree;
        },
        commit: async () => {
          const result = await removePage({ variables: { id: id as PageId } });
          invariant(
            result.data?.removePage.__typename === "RemovePage",
            "TODO: error handling",
          );

          // 削除できたことを確かめてから補充する
          await ensurePageExists(removedTree);
        },
        errorMessage: "ページの削除に失敗しました",
      });
    },
    [ensurePageExists, optimistic, removePage],
  );

  const onMove: SortableTreeProps["onMove"] = useCallback(
    async (fromItem, target) => {
      const moveTarget = ((): MoveTarget => {
        switch (target.type) {
          case "parent":
            return { type: MoveTargetType.Parent, id: target.id as PageId };
          case "siblingParent":
            return {
              type: MoveTargetType.SiblingParent,
              id: target.id as PageId,
            };
          case "siblingChild":
            return {
              type: MoveTargetType.SiblingChild,
              id: target.id as PageId,
            };
        }
      })();

      await optimistic({
        apply: (prev) => moveNode(prev, fromItem.id, target),
        commit: async () => {
          const result = await movePage({
            variables: { id: fromItem.id as PageId, target: moveTarget },
          });
          invariant(
            result.data?.movePage.__typename === "MovePage",
            "TODO: error handling",
          );
        },
        errorMessage: "ページの移動に失敗しました",
      });
    },
    [movePage, optimistic],
  );

  return (
    <SortableTree
      tree={tree}
      onClickCollapse={onClickCollapse}
      onClickAddRoot={onClickAddRoot}
      onClickAddChild={onClickAddChild}
      onClickRename={onClickRename}
      onClickDelete={onClickDelete}
      onMove={onMove}
    />
  );
});
