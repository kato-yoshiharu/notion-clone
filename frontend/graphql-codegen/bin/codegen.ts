import type { CodegenConfig } from "@graphql-codegen/cli";

import { configScalars, defineScalarType } from "../lib/scalar";

const config: CodegenConfig = {
  overwrite: true,
  schema: "http://localhost:8080",
  documents: "src/**/*.graphql.ts",
  generates: {
    "src/graphql/generated/index.ts": {
      plugins: [
        "typescript",
        "typescript-operations",
        "typescript-react-apollo",
        { add: { content: defineScalarType() } },
      ],
      config: {
        withHooks: true,
        scalars: configScalars(),
      },
    },
  },
};

export default config;
