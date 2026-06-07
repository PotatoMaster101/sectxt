import pluginVue from 'eslint-plugin-vue';

export default [
  {
    ignores: ['node_modules/**', 'dist/**'],
  },
  ...pluginVue.configs['flat/essential'],

  {
    files: ['**/*.js', '**/*.vue'],
    rules: {
      'vue/multi-word-component-names': 'off', // Example: disable multi-word requirement
    },
  },
];
