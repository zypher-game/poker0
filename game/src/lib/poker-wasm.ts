import { toBeHex } from 'ethers';
import { sleep } from 'src/utils';

// const pokerWorker = new Worker(new URL('src/workers/poker.ts', import.meta.url));
// pokerWorker.onmessage = (e) => {
//   if (!e.data) return;
//   if (typeof e.data !== 'object') return;
//   const { requestId, from, data, error } = e.data;
//   if (from !== 'poker') return;
//   if (!requestId) return;
//   const cb = cacheMap.get(requestId);
//   if (!cb) return;
//   cb({ data, error });
//   cacheMap.delete(requestId);
// };

// let id = 0;
// const cacheMap = new Map<number, (data: any) => any>();
// export const WorkerShuffle = (action: 'shuffle_cards' | 'Mounted', data: any): Promise<{ data: any; error: Error }> => {
//   const requestId = ++id;
//   return new Promise((resolve) => {
//     cacheMap.set(requestId, resolve);
//     pokerWorker.postMessage({ to: 'poker', requestId, action, data });
//   });
// };

class PokerWasm {
  mounted!: Promise<typeof import('src/wasm/poker_wasm')>;

  sfCache: any = {};

  constructor() {
    this.mounted = this.mount();
  }

  private async mount(): Promise<any> {
    try {
      const poker = new Promise<typeof import('src/wasm/poker_wasm')>(async (resolve) => {
        const zs = await import('src/wasm/poker_wasm');
        resolve(zs);
      });
      this.mounted = poker;
      const zs = await this.mounted;
      return zs;
    } catch (err) {
      await sleep(1000);
      return this.mount();
    }
  }
}

export const pokerWasm = new PokerWasm();
