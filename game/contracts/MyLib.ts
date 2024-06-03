import fs from 'fs';
import path from 'path';
import { HardhatRuntimeEnvironment } from 'hardhat/types';

const cacheDir = path.join(__dirname, './config');
if (!fs.existsSync(cacheDir)) fs.mkdirSync(cacheDir);

class MyLib {
  contract: Record<string, string> = {};

  chainId = 0;

  cacheFileDir = '';

  mountedHandler?: Promise<any>;

  hardhat!: HardhatRuntimeEnvironment;

  outputABI = true;

  async mounted(hardhat: HardhatRuntimeEnvironment) {
    this.hardhat = hardhat;
    console.log('chain', hardhat.network.config.chainId);
    if (this.mountedHandler) return this.mountedHandler;
    this.mountedHandler = new Promise(async (resolve) => {
      const accounts = await this.hardhat.ethers.getSigners();
      console.log('accounts 0', accounts[0].address);
      this.chainId = await this.getEnvironment();
      this.cacheFileDir = path.join(cacheDir, `./Config-${this.chainId}.ts`);
      if (fs.existsSync(this.cacheFileDir) && this.chainId !== 31337) {
        this.contract = require(this.cacheFileDir).Contract;
      } else {
        this.contract = {};
      }
      console.log(this.contract);
      await this.saveConfig();
      resolve(null);
    });
    return this.mountedHandler;
  }

  async clear() {
    this.contract = {};
    await this.saveConfig();
  }

  async getEnvironment() {
    let network = await this.hardhat.ethers.provider.getNetwork();
    while (!network) {
      console.log('!ethers.provider.network sleep 1000');
      await sleep(1000);
      network = await this.hardhat.ethers.provider.getNetwork();
    }
    console.log('ethers.provider.network', network.chainId);
    return Number(network.chainId);
  }

  async verify(name: string, constructorArguments: any[]) {
    await this.hardhat.run('verify:verify', { address: this.contract[name], constructorArguments });
  }

  async deployed<T>(name: string | [string, string], deploy: (e: typeof this.hardhat.ethers.getContractFactory) => Promise<T>, verify = false): Promise<T> {
    let target = '';
    if (name instanceof Array) {
      target = name[1];
      name = name[0];
    } else {
      target = name;
    }
    if (this.contract[target]) {
      const WETHFactory = await this.hardhat.ethers.getContractFactory(name);
      return WETHFactory.attach(this.contract[target]) as any as T;
    }
    const res = await deploy(this.hardhat.ethers.getContractFactory);
    this.contract[target] = (res as any).target;
    console.log('deployed', name, target, this.contract[target]);
    this.saveConfig();
    return res;
  }

  async deploy(KeyName: string, SolSourceName: string, args: any[], options = { verify: false }) {
    if (this.contract[KeyName]) {
      // if (options.verify && this.chainId !== 31337) {
      //   await this.hardhat.run('verify:verify', { address: this.contract[KeyName], constructorArguments: args });
      // }
      return this.searchContract(SolSourceName, this.contract[KeyName]);
    }
    try {
      console.log('deploy', KeyName);
      const Creater = await this.hardhat.ethers.getContractFactory(SolSourceName);
      console.log('deploy', KeyName, 'getContractFactory', SolSourceName);
      const creater = await Creater.deploy(...args);
      console.log('deploy', KeyName, 'Creater', args);
      const dev = creater.deploymentTransaction();
      console.log('tx', dev?.hash, dev?.nonce);
      // const contract = await creater.deployed();
      // this.contract[KeyName] = creater.target as string;
      // console.log('deploy', KeyName, SolSourceName, creater.address);
      // this.saveConfig();
      // if (options.verify && this.chainId !== 31337) {
      //   this.hardhat.run('verify:verify', { address: contract.address, constructorArguments: args });
      // }
      // return contract;
    } catch (e) {
      console.error(e, args);
      throw new Error(`${KeyName}(${SolSourceName})`);
    }
  }

  async searchContract(SolSourceName: string, address: string) {
    const Code = await this.hardhat.artifacts.readArtifact(SolSourceName);
    return this.hardhat.ethers.getContractAt(Code.abi, address);
  }

  async submit(handler: any) {
    const tx = await handler;
    await tx.wait();
    return tx;
  }

  async saveConfig() {
    // save config
    fs.writeFileSync(
      this.cacheFileDir,
      `// update: ${new Date().toLocaleString()}. network: ${this.chainId}
export const Contract = ${JSON.stringify(this.contract, null, 2)}`,
    );
    const ContractNames: string[] = [];
    for (const KeyName in this.contract) {
      try {
        const Code = await this.hardhat.artifacts.artifactExists(KeyName);
        if (Code) {
          ContractNames.push(KeyName);
        }
      } catch (e) {
        //
      }
    }
  }
}

export const myLib = new MyLib();

export function sleep(t = 100) {
  return new Promise((resolve) => setTimeout(resolve, t));
}
