import "destyle.css";
import "@/styles/global.css";

import { ApolloProvider } from "@apollo/client";
import type { AppProps } from "next/app";

import { PageTreeContextProvider } from "@/global-states/page-tree";
import { client } from "@/graphql";

const App = ({ Component, pageProps }: AppProps): JSX.Element => {
  return (
    <ApolloProvider client={client}>
      <PageTreeContextProvider>
        <Component {...pageProps} />
      </PageTreeContextProvider>
    </ApolloProvider>
  );
};

export default App;
