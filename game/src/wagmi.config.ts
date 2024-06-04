import { createConfig, http } from 'wagmi';
import { opBNBTestnet } from 'wagmi/chains';
import { Chain, getDefaultConfig } from '@rainbow-me/rainbowkit';

const projectId = 'bc467c124a7a7a8ce06a41ef40b1b842';

export const SupportNetworks = [opBNBTestnet] as const;
export const SupportNetworksId = SupportNetworks.map((a) => a.id);
export type ISupportNetworksId = (typeof SupportNetworksId)[number];
export const isSupportNetwork = (chain: Chain): chain is (typeof SupportNetworks)[number] => {
  return !!SupportNetworksId.find((id) => id === chain.id);
};

SupportNetworks.forEach((chain) => {
  (chain as any).iconUrl = `https://zypher-static.s3.amazonaws.com/lib/public/chain/${chain.id}.svg`;
});

export const RainbowKitConfig = getDefaultConfig({
  appName: 'PokerZero',
  projectId,
  chains: SupportNetworks,
  ssr: true,
});

export const ChainConfig = {
  [opBNBTestnet.id]: {
    Token: '0x044Ee4a9c25949672214e6D17eA646B2e850e096',
    SimpleGame: '0x7b90EF8E43C696737dc5111A718E404Ca7849168',
    PokerApi: '0x03f1c5a5AC79dFffCD2Af8184b6EFf431b0243b4',
    z4Ws: 'wss://poker0.zypher.dev/ws',
    z4Http: 'https://poker0.zypher.dev/rpc',
  } as const,
};
