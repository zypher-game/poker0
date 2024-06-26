import { defineConfig } from '@wagmi/cli';
import { hardhat } from '@wagmi/cli/plugins';

export default defineConfig({
  out: '../src/wagmi.abi.ts',
  contracts: [],
  plugins: [
    hardhat({
      include: ['SimpleGame.json', 'PokerApi.json'],
      project: '.',
    }),
  ],
});
