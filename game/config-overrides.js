const path = require('path');
const fs = require('fs');

module.exports = function override(config, env) {
  config.resolve.extensions.push('.wasm');
  config.module.rules.forEach((rule) => {
    rule.oneOf?.map((oneOf) => {
      if (oneOf.type === 'asset/resource') {
        oneOf.exclude.push(/\.wasm$/);
      }
    });
  });
  config.experiments = Object.assign({}, config.experiments, {
    // outputModule: true,
    syncWebAssembly: true,
    topLevelAwait: true,
    // asyncWebAssembly: true,
  });
  // Add a dedicated loader for them
  // config.module.rules.push({
  //   test: wasmExtensionRegExp,
  //   include: path.resolve(__dirname, 'src'),
  //   use: [{ loader: require.resolve('wasm-loader'), options: {} }],
  // });
  return config;
};
