import { solidityPackedKeccak256 } from 'ethers';
import { useEffect, useMemo } from 'react';
import { atom, selector, useRecoilState, useSetRecoilState } from 'recoil';
import { useWallet } from 'src/hook/useWallet';
import { localStorageEffect } from 'src/utils';
import { ChainConfig, ISupportNetworksId, RainbowKitConfig, SupportNetworks, isSupportNetwork } from 'src/wagmi.config';
import { useAccount, useSignTypedData, useWalletClient } from 'wagmi';
import { createSiweMessage } from 'viem/siwe';

export const stateLoginState = atom({
  key: 'stateLoginState',
  default: {} as Record<string, string>,
  effects_UNSTABLE: [localStorageEffect('stateLoginState')],
});

export const getSaltBySaltAndToken = (salt: bigint, token: string, chainId: number) => {
  return solidityPackedKeccak256(['uint256', 'string', 'uint256'], [salt, token, chainId]);
};

export const useLoginToken = () => {
  const [auth, _auth] = useRecoilState(stateLoginState);
  const wallet = useWalletClient();

  return useMemo(() => {
    if (!wallet.data) return;
    const address = wallet.data.account.address;
    const chainId = wallet.data.chain.id;
    if (!address || !chainId) return;
    return async (salt: bigint) => {
      const authToken = auth[address];
      if (authToken) return getSaltBySaltAndToken(salt, authToken, chainId);
      const message = createSiweMessage({
        address,
        chainId: 1,
        domain: location.host,
        nonce: 'InitAccount',
        uri: location.origin,
        version: '1',
      });
      const sign = await wallet.data.signMessage({ message });
      const token = solidityPackedKeccak256(['address', 'string'], [address, sign]);
      _auth((v) => ({ ...v, [address]: token }));
      return getSaltBySaltAndToken(salt, token, chainId);
    };
  }, [wallet.data?.account.address, auth]);
};
