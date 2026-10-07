import {
  Dispatch,
  SetStateAction,
  createContext,
  useContext,
  useState,
} from "react";
import { Tree } from "react-notion-sortable-tree";

export type PageTree = Tree<{ title: string }>;

interface PageTreeContextValue {
  pageTree: PageTree;
  setPageTree: Dispatch<SetStateAction<PageTree>>;
}

const PageTreeContext = createContext<PageTreeContextValue>({
  pageTree: [],
  setPageTree: () => {},
});

export const PageTreeContextProvider: React.FC<{
  children: React.ReactNode;
}> = (props) => {
  const [pageTree, setPageTree] = useState<PageTree>([]);

  return (
    <PageTreeContext.Provider value={{ pageTree, setPageTree }}>
      {props.children}
    </PageTreeContext.Provider>
  );
};

export const usePageTree = () => useContext(PageTreeContext);
