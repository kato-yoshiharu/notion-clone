import { invariant } from "@suimenkathemove/utils";
import { NextPage } from "next";
import { useCallback, useMemo } from "react";

import { PagePagePresenter, PagePagePresenterProps } from "./presenter";

import {
  PageId,
  useGetPageInPagePageQuery,
  useListAncestorPagesQuery,
  useUpdatePageMutation,
} from "@/graphql/generated";
import { useRouterQuery } from "@/hooks/use-router-query";

export const PagePage: NextPage = () => {
  const routerQuery = useRouterQuery(["page-id"]);

  const getPageInPagePageResult = useGetPageInPagePageQuery(
    routerQuery.isReady
      ? { variables: { id: routerQuery.query["page-id"] as PageId } }
      : { skip: true },
  );

  const [updatePage] = useUpdatePageMutation();
  const updateTitle = useCallback(
    async (title: string) => {
      invariant(routerQuery.isReady, "routerQuery is ready");
      await updatePage({
        variables: {
          id: routerQuery.query["page-id"] as PageId,
          updatePage: { title },
        },
      });
    },
    [routerQuery.isReady, routerQuery.query, updatePage],
  );
  const updateText = useCallback(
    async (text: string) => {
      invariant(routerQuery.isReady, "routerQuery is ready");
      await updatePage({
        variables: {
          id: routerQuery.query["page-id"] as PageId,
          updatePage: { text },
        },
      });
    },
    [routerQuery.isReady, routerQuery.query, updatePage],
  );

  const listAncestorPagesResult = useListAncestorPagesQuery(
    routerQuery.isReady
      ? { variables: { id: routerQuery.query["page-id"] as PageId } }
      : { skip: true },
  );
  // TODO: error handling?
  const ancestors = useMemo<PagePagePresenterProps["ancestors"]>(
    () =>
      listAncestorPagesResult.data?.listAncestorPages.__typename === "ListPages"
        ? listAncestorPagesResult.data.listAncestorPages.items.map((item) => ({
            id: item.id,
            name: item.title,
          }))
        : [],
    [listAncestorPagesResult.data?.listAncestorPages],
  );

  const title =
    getPageInPagePageResult.data?.getPage.__typename === "Page"
      ? getPageInPagePageResult.data.getPage.title
      : "";
  const text =
    getPageInPagePageResult.data?.getPage.__typename === "Page"
      ? getPageInPagePageResult.data.getPage.text
      : "";

  return (
    <PagePagePresenter
      ancestors={ancestors}
      title={title}
      onChangeTitle={updateTitle}
      text={text}
      onChangeText={updateText}
    />
  );
};
