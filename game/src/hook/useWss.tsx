import { ethers } from 'ethers';
import { useEffect, useMemo, useRef } from 'react';
import { Z4WebSocket } from 'src/lib/wss';
import { zeroAddress } from 'viem';

export interface Z4WssMsg {
  gid: number;
  id: number;
  jsonrpc: string;
  method: string;
  result: any;
  error?: { code: number; message: string };
}

const cache: Record<
  string,
  {
    provider: Z4WebSocket;
    mounted: Promise<any>;
    deps: Array<(msg: Z4WssMsg) => any>;
  }
> = {};
export const useWss = (url: string, deps: any[], callback?: (msg: Z4WssMsg) => any) => {
  const cb = useRef(callback);
  cb.current = callback;
  useEffect(() => {
    if (!url) return;
    if (!cache[url]) {
      let provider = new Z4WebSocket(url);
      const deps: Array<(msg: Z4WssMsg) => any> = [];
      const mounted = new Promise((resolve) => {
        provider.bus.on('open', resolve);
        provider.bus.on('close', (evt) => {
          console.log('ws', 'close', evt);
        });
        provider.bus.on('error', (evt) => {
          console.log('ws', 'error', evt);
        });
        provider.bus.on('message', (...args) => {
          // console.log('onmessage', args);
          args.forEach((msg: any) => {
            deps.forEach((dep) => dep(msg));
          });
        });
        // provider.onmessage = (...args) => {
        //   console.log('onmessage', args);
        //   args.forEach((msg) => {
        //     const data = JSON.parse(msg.data);
        //     deps.forEach((dep) => dep(data));
        //   });
        // };
      });
      cache[url] = { provider, mounted, deps };
    }
    const wss = cache[url];
    const handler = (msg: Z4WssMsg) => {
      try {
        cb.current?.(msg);
      } catch (e) {
        console.error('useWss callback', e);
      }
    };
    wss.deps.push(handler);
    return () => {
      const index = wss.deps.indexOf(handler);
      if (index !== -1) wss.deps.splice(index, 1);
      if (wss.deps.length === 0) {
        wss.provider.close();
        delete cache[url];
      }
      console.log('wss', url, index, wss.deps, cache[url]);
    };
  }, [url, ...deps]);

  return useMemo(() => {
    return {
      send: async (body: Partial<{ peer: `0x${string}`; jsonrpc: string; id: number; gid: number; method: string; params: any[] }>) => {
        const wss = cache[url];
        if (!wss) return;
        await wss.mounted;
        wss.provider.send(Object.assign({ peer: zeroAddress, jsonrpc: '2.0', id: 0, gid: 4, params: [] }, body));
      },
      request: async (body: Partial<{ peer: `0x${string}`; jsonrpc: '2.0'; gid: number; method: string; params: any[] }>, timeout = 3000) => {
        return new Promise(async (resolve, reject) => {
          const wss = cache[url];
          if (!wss) return;
          await wss.mounted;
          const id = Date.now();
          let returned = false;
          const timer = setTimeout(() => {
            if (returned) return;
            returned = true;
            resolve(null);
          }, timeout);
          const hook = (res: any) => {
            if (res.id !== id) return;
            wss.provider.bus.off('message', hook);
            if (returned) return;
            returned = true;
            clearTimeout(timer);
            resolve(res);
          };

          wss.provider.bus.on('message', hook);
          wss.provider.send(Object.assign({ peer: zeroAddress, jsonrpc: '2.0', id, gid: 4, params: [] }, body));
        });
      },
    };
  }, [url, ...deps]);
};
