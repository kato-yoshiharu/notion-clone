/** @type {import('next').NextConfig} */
const nextConfig = {
  // SSRを使っていないため静的に書き出し、Cloudflare WorkersのStatic Assetsで配信する。
  output: "export",
  reactStrictMode: true,
  swcMinify: true,
  compiler: {
    styledComponents: true,
  },
};

module.exports = nextConfig;
