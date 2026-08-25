import { memo, useCallback, useEffect, useRef } from "react";

import { Container, Content, HeaderContainer, Text, Title } from "./styles";

import { Page } from "@/graphql/generated";
import { useDebouncedCallback } from "@/hooks/use-debounced-callback";

const DEBOUNCE_MS = 500;

export interface PageContentProps {
  title: Page["title"];
  onChangeTitle: (title: string) => Promise<void>;
  text: Page["text"];
  onChangeText: (text: string) => Promise<void>;
}

export const PageContent = memo((props: PageContentProps) => {
  const { onChangeTitle, onChangeText } = props;

  const titleElRef = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    const el = titleElRef.current;
    if (el == null) return;
    el.textContent = props.title;
  }, [props.title]);
  const { debounced: sendTitle, flush: flushTitle } = useDebouncedCallback(
    (value: string) => {
      void onChangeTitle(value);
    },
    DEBOUNCE_MS,
  );
  const onInputTitle: React.FormEventHandler<HTMLHeadingElement> = useCallback(
    (event) => {
      sendTitle(event.currentTarget.textContent ?? "");
    },
    [sendTitle],
  );

  const textElRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = textElRef.current;
    if (el == null) return;
    el.textContent = props.text;
  }, [props.text]);
  const { debounced: sendText, flush: flushText } = useDebouncedCallback(
    (value: string) => {
      void onChangeText(value);
    },
    DEBOUNCE_MS,
  );
  const onInputText: React.FormEventHandler<HTMLDivElement> = useCallback(
    (event) => {
      sendText(event.currentTarget.textContent ?? "");
    },
    [sendText],
  );

  return (
    <Container>
      <Content>
        <HeaderContainer>
          <Title
            contentEditable
            suppressContentEditableWarning
            onInput={onInputTitle}
            // フォーカスを外した時点で未送信分を送る
            onBlur={flushTitle}
            ref={titleElRef}
          />
        </HeaderContainer>
        <Text
          contentEditable
          suppressContentEditableWarning
          onInput={onInputText}
          onBlur={flushText}
          ref={textElRef}
        />
      </Content>
    </Container>
  );
});
