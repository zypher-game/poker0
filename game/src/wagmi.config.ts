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
    Token: '0x17C30AA61ABc0c4dc7BD8C836A595d19Df6cf3AD',
    SimpleGame: '0xB38c18E85BCbAc851588dc5776658d506511A8d5',
    PokerApi: '0x03f1c5a5AC79dFffCD2Af8184b6EFf431b0243b4',
    z4Ws: 'wss://poker0.zypher.dev/ws',
    z4Http: 'https://poker0.zypher.dev',
  } as const,
};
