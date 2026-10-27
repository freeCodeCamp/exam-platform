import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import { defineConfig, globalIgnores, type Config } from "eslint/config";

type Rules = Config["rules"];

const tsConfig = [js.configs.recommended, tseslint.configs.recommendedTypeChecked];
const reactConfig = [react.configs.flat.recommended!, react.configs.flat["jsx-runtime"]!];
const tsRules: Rules = {
  "@typescript-eslint/no-misused-promises": [
    "error",
    {
      checksVoidReturn: {
        attributes: false,
      },
    },
  ],
};

const tsLanguageOptions = {
  languageOptions: {
    parserOptions: {
      projectService: true,
    },
  },
};

export default defineConfig([
  globalIgnores(["dist/", "**/.astro/"]),
  { files: ["**/*.ts"], extends: tsConfig, ...tsLanguageOptions },
  {
    files: ["**/*.tsx"],
    settings: {
      react: {
        version: "19",
      },
    },
    extends: [...reactConfig, ...tsConfig],
    ...tsLanguageOptions,
    rules: tsRules,
  },
]);
