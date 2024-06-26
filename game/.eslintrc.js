const path = require('path');

module.exports = {
  env: {
    browser: true,
    es2021: true,
  },
  extends: ['plugin:react/recommended'],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaFeatures: {
      jsx: true,
    },
    ecmaVersion: 12,
    sourceType: 'module',
  },
  globals: {
    gtag: true,
  },
  plugins: ['react', '@typescript-eslint'],
  rules: {
    'dot-notation': 'off',
    'no-continue': 'off',
    'no-case-declarations': 'off',
    'import/no-extraneous-dependencies': 'off',
    'react/function-component-definition': 'off',
    'no-debugger': 'off',
    'prefer-destructuring': 'off',
    'no-loop-func': 'off',
    camelcase: 'off',
    'react/jsx-pascal-case': 'off',
    'no-alert': 'off',
    'no-lonely-if': 'off',
    'no-async-promise-executor': 'off',
    'react/jsx-boolean-value': 'off',
    'react/no-array-index-key': 'off',
    'no-await-in-loop': 'off',
    'no-restricted-globals': 'off',
    'no-underscore-dangle': 'off',
    'react-hooks/exhaustive-deps': 'off',
    'no-use-before-define': 'off',
    'consistent-return': 'off',
    radix: 'off',
    'no-param-reassign': 'off',
    'no-var': 'error',
    '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
    'react/button-has-type': 'off',
    'react/prop-types': 'off',
    'react/sort-comp': 'off',
    'import/extensions': 'off',
    'react/jsx-filename-extension': [1, { extensions: ['.js', '.jsx', '.ts', '.tsx'] }],
    'no-useless-constructor': 'off',
    'no-unused-vars': 'off',
    '@typescript-eslint/no-useless-constructor': 'error',
    'no-console': 'off',
    'react/self-closing-comp': 'off',
    'import/no-dynamic-require': 'off',
    'global-require': 'off',
    'arrow-body-style': 'off',
    'jsx-a11y/alt-text': 'off',
    'jsx-a11y/click-events-have-key-events': 'off',
    'import/prefer-default-export': 'off',
    'jsx-a11y/no-noninteractive-element-interactions': 'off',
    'jsx-a11y/no-static-element-interactions': 'off',
    'react/destructuring-assignment': 'off',
    'react/require-default-props': 'off',
    'react/no-unused-prop-types': 'off',
    'prettier/prettier': 'off',
    'react/jsx-no-bind': 'off',
    'react/jsx-props-no-spreading': 'off',
    'no-plusplus': 'off',
    'no-shadow': 'off',
    'no-template-curly-in-string': 'off',
    'no-restricted-syntax': 'off',
    'guard-for-in': 'off',
    'spaced-comment': 'off',
  },
  overrides: [
    {
      files: ['**/*.d.ts'],
      rules: {
        'import/no-duplicates': 0,
      },
    },
  ],
  settings: {
    'import/resolver': {
      node: {
        extensions: ['.js', '.less', '.scss', '.jsx', '.tsx', '.ts', '.json', '.jsonc', '.wasm'],
      },
      alias: {
        extensions: ['.js', '.less', '.scss', '.jsx', '.tsx', '.ts', '.json', '.jsonc', '.wasm'],
      },
    },
  },
};
