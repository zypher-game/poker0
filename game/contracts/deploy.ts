import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { task, types } from 'hardhat/config';
import hardhat from 'hardhat';
import fs from 'fs';
import path from 'path';
import { getCreate2Address, hexToSignature, keccak256 } from 'viem';
import { ethers, solidityPackedKeccak256 } from 'ethers';
import { myLib } from './MyLib';

/**
  cd ./contracts/ && npx hardhat run ./deploy.ts --network opbnbtest
 */

task('deploy', '').setAction(async (args, hre) => {
  await hre.run('compile');
  main(hre);
});

export async function main(hardhat: HardhatRuntimeEnvironment) {
  // myLib.outputABI = false;
  await myLib.mounted(hardhat);
  const accounts = await hardhat.ethers.getSigners();

  // const Multicall3 = await myLib.deployed('Multicall3', (f) => f('Multicall3').then((a) => a.deploy()));

  const Token = await myLib.deployed('PokerApi', (f) => f('PokerApi').then((a) => a.deploy()));
  // const Token = await myLib.deployed('Token', (f) => f('Token').then((a) => a.deploy(1e10)));
  // const RoomMarket = await myLib.deployed('RoomMarket', (f) => f('RoomMarket').then((a) => a.deploy(Token.target, 10000, 100)));
  // const Demo = await myLib.deployed('Demo', (f) => f('Demo').then((a) => a.deploy(RoomMarket.target)));
  process.exit(0);
}
hardhat.run('deploy');
