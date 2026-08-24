import { memo } from "react";

import { Header, HeaderProps } from "@/components/domains/header";
import {
  PageContent,
  PageContentProps,
} from "@/components/domains/page-content";
import { Sidebar } from "@/components/domains/sidebar";
import { Layout } from "@/components/layout";

export type PagePagePresenterProps = {
  ancestors: HeaderProps["ancestors"];
  title: PageContentProps["title"];
  onChangeTitle: PageContentProps["onChangeTitle"];
  text: PageContentProps["text"];
  onChangeText: PageContentProps["onChangeText"];
};

export const PagePagePresenter = memo((props: PagePagePresenterProps) => {
  return (
    <Layout
      sidebar={<Sidebar />}
      header={<Header ancestors={props.ancestors} />}
      main={
        <PageContent
          title={props.title}
          onChangeTitle={props.onChangeTitle}
          text={props.text}
          onChangeText={props.onChangeText}
        />
      }
    />
  );
});
