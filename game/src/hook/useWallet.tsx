import { useRecoilValue } from 'recoil';
import { useAccount, useChains, useClient, useConnect, usePublicClient, useTransactionConfirmations, useWriteContract } from 'wagmi';
import { useAddRecentTransaction, useChainModal, useConnectModal } from '@rainbow-me/rainbowkit';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';
import { TransactionReceipt, zeroAddress } from 'viem';
import { stateAppWallet } from 'src/states/wallet';
import { GlobalVar } from 'src/constants';
import { is0xString } from 'src/utils';

export const useWallet = () => {
  const appWallet = useRecoilValue(stateAppWallet);
  const acc = useAccount();
  const chainModal = useChainModal();
  const connModal = useConnectModal();
  const write = useWriteContract();
  const publicClient = usePublicClient();
  const addTx = useAddRecentTransaction();
  const hasError = () => {
    if (!acc.address) {
      connModal.openConnectModal?.();
      return true;
    }
    if (!acc.chain) {
      chainModal.openChainModal?.();
      return true;
    }
    return false;
  };
  const res = {
    appWallet,
    acc,
    address: acc.address || zeroAddress,
    write,
    hasError,
    publicClient,
    handler<T>(options: { setLoading?: (b: boolean) => any; action: string; callback: (arg: T) => Promise<any>; onSuccess?: (res: TransactionReceipt) => any; onError?: (err: any) => any }) {
      let pending = false;
      return async (arg: T) => {
        console.log('requestTx', arg);
        if (pending) return;
        if (!acc.address) return connModal.openConnectModal?.();
        if (!acc.chain) return chainModal.openChainModal?.();
        if (!publicClient) return;
        const key = Date.now().toString();
        pending = true;
        // const closeLoading = GlobalVar.message.loading(options.action);
        try {
          console.log('... req tx');
          options.setLoading?.(true);
          const hash = await options.callback(arg);
          console.log('... req txdd');
          if (!hash) return;
          if (!is0xString(hash)) return hash;
          addTx({ hash, description: options.action });
          const res = await publicClient.waitForTransactionReceipt({ hash, confirmations: 1 });
          options.onSuccess?.(res);
          return res;
        } catch (e) {
          const msg = String(e);
          console.error('... req tx', e);
          if (msg.match('User rejected the request')) return GlobalVar.notification.destroy(key);
          options.onError?.(e);
          return msg;
        } finally {
          options.setLoading?.(false);
          console.log('... req tx finally');
          pending = false;
          // closeLoading();
        }
      };
    },
  };
  return res;
};

const PageStyle = styled.div``;
