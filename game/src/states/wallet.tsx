import { useEffect } from 'react';
import { atom, selector, useSetRecoilState } from 'recoil';
import { localStorageEffect } from 'src/utils';
import { ChainConfig, ISupportNetworksId, RainbowKitConfig, SupportNetworks, isSupportNetwork } from 'src/wagmi.config';
import { useAccount } from 'wagmi';

const defaultChain = RainbowKitConfig.chains[0] as any as (typeof SupportNetworks)[0];

export const stateAppWallet = atom({
  key: 'stateAppWallet',
  default: {
    chain: defaultChain as (typeof RainbowKitConfig.chains)[number],
    config: ChainConfig[defaultChain.id] as (typeof ChainConfig)[ISupportNetworksId],
  },
});

export const useAppWallet = () => {
  const _state = useSetRecoilState(stateAppWallet);
  const acc = useAccount();

  useEffect(() => {
    const chain = acc.chain;
    if (!chain) return;
    if (!isSupportNetwork(chain)) return;
    const config = ChainConfig[chain.id];
    _state({ chain, config });
  }, [acc.chain?.id]);
};

export const stateUserZkss = atom({
  key: 'stateUserZkss',
  default: {} as Record<string, { roomId?: number; pk: `0x${string}`; peer: `0x${string}`; pkxy: [`0x${string}`, `0x${string}`]; sk: `0x${string}` }>,
  effects_UNSTABLE: [localStorageEffect('stateUserZkss')],
});
