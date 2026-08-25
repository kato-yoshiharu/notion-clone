import { invariant } from "@suimenkathemove/utils";
import { useRouter } from "next/router";
import { useCallback } from "react";

import { PageTree, usePageTree } from "@/global-states/page-tree";
import { PageId, useAddPageMutation } from "@/graphql/generated";
import { routes } from "@/routes";

/**
 * 1ページも無い状態は許容しないため、そうなっていたら代わりに1ページ作って遷移する。
 */
export const useEnsurePageExists = () => {
  const { setPageTree } = usePageTree();

  const [addPage] = useAddPageMutation();

  const router = useRouter();

  return useCallback(
    async (pageTree: PageTree) => {
      if (pageTree.length !== 0) return;

      const id = crypto.randomUUID() as PageId;
      setPageTree((prev) =>
        prev.concat({
          id,
          children: [],
          collapsed: true,
          data: {
            title: "",
          },
        }),
      );

      const result = await addPage({
        variables: { parentId: null, addPage: { id, title: "", text: "" } },
      });
      invariant(
        result.data?.addPage.__typename === "Page",
        "TODO: error handling",
      );

      await router.push(routes.notion.page.show(id));
    },
    [addPage, router, setPageTree],
  );
};
