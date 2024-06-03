import { atom, useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { zeroAddress } from 'viem';
import { stateAppWallet } from 'src/states/wallet';
import { useEffect, useRef, useState } from 'react';
import { ethers } from 'ethers';
import { useWss } from 'src/hook/useWss';
import { GlobalVar } from 'src/constants';

export const PageIndexState = atom({
  key: 'PageIndexState',
  default: {
    rooms: [] as Array<{
      players: `0x${string}`[];
      room: number;
    }>,
    // nextRoomId: 10000n,
  },
});

export const usePageIndexState = () => {
  const { chain, config } = useRecoilValue(stateAppWallet);
  const _pageState = useSetRecoilState(PageIndexState);
  const [update, _update] = useState(0);
  const query = useRef(false);
  // const z4Wss = useWss(config.z4Ws, [], (msg) => {
  //   if (msg.method !== 'room_market') return;
  //   console.log('room_market', msg.result);
  //   _pageState({ rooms: msg.result });
  // });

  useEffect(() => {
    if (query.current) return;
    query.current = true;
    const body = { gid: 4, method: 'room_market', params: [config.SimpleGame], peer: zeroAddress, jsonrpc: '2.0', id: 0 };
    const queryData = async () => {
      const res = await fetch(config.z4Http, { method: 'POST', body: JSON.stringify(body) });
      if (res.status !== 200) return;
      const data = await res.json();
      if (data.error) return GlobalVar.notification.error({ message: `Code: ${data.error.code}`, description: data.error.message });
      _pageState({ rooms: data.result });
    };
    let live = true;
    queryData();
    const timer = setInterval(() => {
      if (!live) return;
      queryData();
    }, 10000);
    return () => {
      live = false;
      query.current = false;
      clearInterval(timer);
    };
  }, [update]);

  return {
    refetch: () => _update((v) => ++v),
  };
};
