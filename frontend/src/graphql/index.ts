import { ApolloClient, InMemoryCache } from "@apollo/client";

export const client = new ApolloClient({
  // 本番は同一オリジンの`/graphql`をWorkerがLambdaへ中継する。
  uri: process.env.NEXT_PUBLIC_GRAPHQL_URL ?? "http://localhost:8080",
  cache: new InMemoryCache(),
  defaultOptions: {
    watchQuery: {
      fetchPolicy: "network-only",
    },
    mutate: {
      refetchQueries: "active",
    },
  },
});
