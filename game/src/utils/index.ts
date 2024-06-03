import { AtomEffect } from 'recoil';

export const is0xString = (tx: any): tx is `0x${string}` => {
  if (!tx) return false;
  if (typeof tx !== 'string') return false;
  if (!tx.match(/^0x/)) return false;
  return true;
};

export const localStorageEffect = <T>(key: string) => {
  const fun: AtomEffect<T> = ({ setSelf, onSet }) => {
    const savedValue = localStorage.getItem(key);
    if (savedValue != null) {
      setSelf(JSON.parse(savedValue));
    }
    onSet((newValue) => {
      localStorage.setItem(key, JSON.stringify(newValue));
    });
  };
  return fun;
};

export const sleep = (t = 200) => new Promise((r) => setTimeout(r, t));

// https://openchain.xyz/signatures?query=0xfb8f41b2
export const OpenChainSignature: Record<string, string> = {
  '0xfb8f41b2': 'ERC20InsufficientAllowance(address,uint256,uint256)',
  '0xe450d38c': 'ERC20InsufficientBalance(address,uint256,uint256)',
  '0xec442f05': 'ERC20InvalidReceiver(address)',
};

export const errorParse = (e: any) => {
  console.error('errorParse', e);
  if (!e) return 'unknown error';
  const res = parseInner(e);
  // gtag('event', 'exception', { description: res.slice(0, 50), fatal: false });
  return res;
};

const parseInner = (e: any) => {
  // The contract function "mint" reverted with the following signature:\n0xfb8f41b2'
  if (e && 'name' in e && e.name === 'ContractFunctionExecutionError' && e.shortMessage) {
    const signature = e.shortMessage.replace(/(.*)signature:\n/, '');
    if (signature in OpenChainSignature) return OpenChainSignature[signature];
  }
  let msg = String(e);

  if (typeof e === 'object') {
    if (typeof e.data === 'object' && e.data.message) {
      msg = String(e.data.message);
    } else if (typeof e.message === 'string') {
      msg = String(e.message);
    } else if (typeof e.reason === 'string') {
      msg = e.reason;
    }
  }
  msg = msg.replace(/TransactionExecutionError: /, '');
  msg = msg.replace(/\n\nRequest Arguments:([\w\W]*)/, '');
  msg = msg.replace(/\n\nRaw Call Arguments:([\w\W]*)/, '');

  msg = msg.replace(/Contract Call:([\w\W]*)/, '');

  // TransactionExecutionError: User rejected the request. Request Arguments: from: 0x34df25eae393ab
  msg = msg.replace(/(.*){\\"code\\":-32000,\\"message\\":\\"(.*?)\\"}}([\w\W]*)/, '$2');
  if (msg.match(/underlying network changed/)) msg = 'underlying network changed';
  msg = msg.replace(/(.*)execution reverted: (.*?)"(.*)/, '$2').replace(/\\$/, '');
  msg = msg.replace(/VM Exception while processing transaction: /, '');
  msg = msg.replace(/\(action="sendTransaction", transaction=(.*)/, '');
  msg = msg.replace(/Error: /, '');
  if (msg.match(/SilenceError:/)) return '';
  msg = msg.slice(0, 500);
  msg = msg.replace(/\[ See: https:\/\/links.ethers.org\/(.*)/, '');
  if (msg.match('ERC20: insufficient allowance')) return 'ERC20InsufficientAllowance(address,uint256,uint256)';
  return msg;
};
