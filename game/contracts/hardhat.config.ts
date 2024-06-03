import { HardhatUserConfig, task } from 'hardhat/config';
import '@nomicfoundation/hardhat-ethers';
import '@nomiclabs/hardhat-etherscan';
import 'hardhat-gas-reporter';
import '@typechain/hardhat';
import fs from 'fs';
import dotenv from 'dotenv';
import path from 'path';
import { ethers } from 'ethers';
import { opBNBTestnet } from 'viem/chains';

dotenv.config({ path: './.env' });

// const newWallet = ethers.Wallet.createRandom().mnemonic?.phrase;
// console.log(encode(newWallet!));

const encode = (str: string) =>
  str
    .split('')
    .map((c) => c.charCodeAt(0) + 1)
    .join('-');
const decode = (str: string) =>
  str
    .split('-')
    .map((s) => String.fromCharCode(parseInt(s) - 1))
    .join('');

const settings = {
  optimizer: {
    enabled: true,
    runs: 1000,
  },
  viaIR: true,
};

const getPk = (file = '.secret', mp = "m/44'/60'/0'/0/0") => {
  let mnemonic = fs.existsSync(file) ? fs.readFileSync(file).toString().trim() : '';
  if (mnemonic.split('-').length > 10) mnemonic = decode(mnemonic);
  if (mnemonic.split('-').length < 10 && mnemonic.match(/ /)) mnemonic = mnemonic.replace(/-/g, ' ');
  if (mnemonic.split('-').length < 10 && !mnemonic.match(/ /)) mnemonic = mnemonic.replace(/-/g, '');
  // console.log('mnemonic', mnemonic.replace(/ /g, '-'));
  let privateKey = '';
  if (mnemonic.match(/ /)) {
    const wallet = ethers.Wallet.fromPhrase(mnemonic);
    privateKey = wallet.derivePath(mp).privateKey;
    // console.log(wallet.address, privateKey);
  } else {
    privateKey = mnemonic;
  }
  return privateKey;
};

const config: HardhatUserConfig = {
  paths: {
    root: path.join(__dirname),
    sources: path.join(__dirname, `./contracts`),
    cache: path.join(__dirname, './cache'),
    artifacts: path.join(__dirname, './artifacts'),
  },
  networks: {
    localhost: {
      url: 'http://127.0.0.1:8545/',
      accounts: {
        // 0x56ab7cb5D77de2E986f189381A96aE1107edcdbD
        mnemonic: 'tail input unhappy sibling slight ethics stick assault when spot tilt together',
      },
      chainId: 31337,
    },
    opbnbtest: {
      url: opBNBTestnet.rpcUrls.default.http[0],
      accounts: [getPk()],
      chainId: opBNBTestnet.id,
    },
    hardhat: {
      accounts: [
        {
          // 0x56ab7cb5D77de2E986f189381A96aE1107edcdbD
          privateKey: '0x3b8d9855a526c70112f95bbef82fc836d38ecf1d18f8ecc0ac2eec3914e4c25c',
          balance: '100000000000000000000',
        },
        {
          // 0x593F57011CC80fEDA0C05Ea8461f470C9440773D
          privateKey: '0xf448fdf9a7e60c5c993d560193924de71e93eb4e7d11cb6bc1b7f32db4a6adcb',
          balance: '100000000000000000000',
        },
      ],
      blockGasLimit: 16777215,
      mining: { auto: false, interval: 1000 },
    },
  },
  solidity: {
    compilers: [
      { version: '0.8.20', settings },
      { version: '0.8.15', settings },
      { version: '0.8.12', settings },
      { version: '0.6.0', settings },
      { version: '0.6.6', settings },
      { version: '0.7.6', settings },
      { version: '0.5.16', settings },
      // { version: '0.4.24', settings },
    ],
  },
  gasReporter: {
    currency: 'ETH',
    gasPrice: 1,
  },
  etherscan: {
    apiKey: {
      mainnet: 'YOUR_ETHERSCAN_API_KEY',
      kovan: 'YOUR_ETHERSCAN_API_KEY',
    },
  },
  typechain: {
    outDir: './types',
    target: 'ethers-v6',
    alwaysGenerateOverloads: false, // should overloads with full signatures like deposit(uint256) be generated always, even if there are no overloads?
    externalArtifacts: ['externalArtifacts/*.json'], // optional array of glob patterns with external artifacts to process (for example external libs from node_modules)
    dontOverrideCompile: false, // defaults to false
  },
};

export default config;
