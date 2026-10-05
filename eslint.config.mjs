import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import hooks from "eslint-plugin-react-hooks";
import prettier from "eslint-config-prettier";

export default tseslint.config(
  { ignores: ["node_modules/**", "dist/**", "src-tauri/**"] },
  js.configs.recommended,
  {
    files: ["**/*.ts", "**/*.tsx"],
    extends: [
      ...tseslint.configs.strictTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: { react, "react-hooks": hooks },
    settings: { react: { version: "19.1" } },
    rules: {
      ...Object.fromEntries(
        Object.entries(hooks.configs.recommended.rules).map(([rule, value]) => [
          rule,
          Array.isArray(value) ? ["error", ...value.slice(1)] : "error",
        ]),
      ),
      // Callable interfaces are an explicit project contract.
      "@typescript-eslint/prefer-function-type": "off",
      "@typescript-eslint/consistent-type-definitions": "off",
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/explicit-function-return-type": [
        "error",
        { allowExpressions: true },
      ],
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "react/jsx-key": "error",
      "react/button-has-type": "error",
      "react/jsx-no-target-blank": "error",
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "Program > VariableDeclaration > VariableDeclarator[init.type='ArrowFunctionExpression']",
          message: "Top-level functions use function declarations.",
        },
      ],
    },
  },
  {
    files: ["src/bridge/generated/**/*.ts", "src/route-tree.gen.ts"],
    rules: {
      "@typescript-eslint/explicit-function-return-type": "off",
      "no-restricted-syntax": "off",
    },
  },
  // Pinned generators emit these casts; handwritten code receives no exception.
  {
    files: ["src/bridge/generated/bindings.ts"],
    rules: {
      "@typescript-eslint/no-unnecessary-type-parameters": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  {
    files: ["src/route-tree.gen.ts"],
    rules: {
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  prettier,
);
