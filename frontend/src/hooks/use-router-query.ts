import { useRouter } from "next/router";
import { useMemo } from "react";

type Query<T extends string> = Record<T, string>;

type UseRouterQueryResult<T extends string> =
  { isReady: false; query: null } | { isReady: true; query: Query<T> };

export const useRouterQuery = <T extends string>(
  keys: T[],
): UseRouterQueryResult<T> => {
  const router = useRouter();

  return useMemo<UseRouterQueryResult<T>>(() => {
    if (!router.isReady) return { isReady: router.isReady, query: null };

    const query = keys.reduce((acc, key) => {
      const value = router.query[key];

      return { ...acc, [key]: Array.isArray(value) ? value[0] : value };
    }, {} as Query<T>);

    return { isReady: router.isReady, query };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [router.isReady, router.query]);
};
