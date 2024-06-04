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
    Token: '0xE2EB8fE74541F7bEB437F2362403216d521Ba394',
    SimpleGame: '0x9DA1371Fd81EF9356AF1B2036F1ed0ac85663669',
    PokerApi: '0x52FdAEE21Fa091f5BA9726810d2E39D80A80876f',
    z4Ws: 'wss://poker0.zypher.dev/ws',
    z4Http: 'https://poker0.zypher.dev/rpc',
  } as const,
};
