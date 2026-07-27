import { NextPage } from "next";
import { useRouter } from "next/router";
import { useEffect } from "react";

import { useListRootPagesQuery } from "@/graphql/generated";
import { routes } from "@/routes";

const Home: NextPage = () => {
  const router = useRouter();
  const listRootPagesResult = useListRootPagesQuery();

  useEffect(() => {
    const data = listRootPagesResult.data;
    if (data == null) return;
    if (data.listRootPages.__typename !== "ListPages") return;
    const firstPage = data.listRootPages.items[0];
    if (firstPage == null) return;
    void router.replace(routes.notion.page.show(firstPage.id));
  }, [listRootPagesResult.data, router]);

  return null;
};

export default Home;
