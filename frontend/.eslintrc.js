module.exports = {
  extends: [
    // "@suimenkathemove/frontend-eslint-config",
    "./node_modules/@suimenkathemove/frontend-eslint-config",
  ],
  ignorePatterns: [
    "src/graphql/generated",
    "src/wasm",
    "worker",
    "out",
    ".next",
    "storybook-static",
    "coverage",
  ],
};
