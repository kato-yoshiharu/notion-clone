/**
 * 静的アセットの配信と、バックエンドへのリバースプロキシを兼ねるWorker。
 *
 * ブラウザから直接Lambdaを叩く形にすると共有シークレットをJSに埋める必要があり
 * 秘密にできないため、APIは必ずこのWorkerを経由させる。
 */

// Envは`wrangler types`が wrangler.jsonc と .dev.vars から生成する。
// バインディングを増やしたら再生成すること。

const API_PATH = "/graphql";

/** 静的書き出しで生成される動的ルートのシェル。 */
const PAGE_SHELL = "/[page-id].html";

async function proxyToBackend(request: Request, env: Env): Promise<Response> {
  const backend = new URL(env.BACKEND_ORIGIN);

  const headers = new Headers(request.headers);
  headers.set("x-origin-secret", env.ORIGIN_SHARED_SECRET);
  // Lambda Function URLはHostヘッダで宛先を判定するため、元の値を残さない。
  headers.delete("host");

  return fetch(
    new Request(backend, {
      method: request.method,
      headers,
      body: request.body,
      redirect: "manual",
    }),
  );
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === API_PATH) {
      // 静的アセットはCloudflareのキャッシュで捌けるため、Lambdaの実行とDBアクセスを伴うAPIだけを制限する。
      const clientIp = request.headers.get("cf-connecting-ip") ?? "unknown";
      const { success } = await env.API_RATE_LIMITER.limit({ key: clientIp });
      if (!success) {
        return new Response("Too Many Requests", { status: 429 });
      }

      return proxyToBackend(request, env);
    }

    const asset = await env.ASSETS.fetch(request);
    if (asset.status !== 404) {
      return asset;
    }

    // `/<page-id>` は事前に列挙できないため、静的アセットには存在しない。
    // シェルを返し、ページIDはクライアント側のルータに解決させる。
    const shell = await env.ASSETS.fetch(
      new Request(new URL(PAGE_SHELL, url.origin)),
    );

    return new Response(shell.body, {
      status: 200,
      headers: shell.headers,
    });
  },
};
