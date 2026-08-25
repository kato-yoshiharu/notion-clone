import { invariant } from "@suimenkathemove/utils";
import { useRouter } from "next/router";
import { useCallback } from "react";

import { PageTree, usePageTree } from "@/global-states/page-tree";
import { useAddPageMutation } from "@/graphql/generated";
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

      const result = await addPage({
        variables: { parentId: null, addPage: { title: "", text: "" } },
      });
      invariant(
        result.data?.addPage.__typename === "Page",
        "TODO: error handling",
      );
      const newPage = result.data.addPage;
      setPageTree((prev) =>
        prev.concat({
          id: newPage.id,
          children: [],
          collapsed: true,
          data: {
            title: "",
          },
        }),
      );

      await router.push(routes.notion.page.show(newPage.id));
    },
    [addPage, router, setPageTree],
  );
};
