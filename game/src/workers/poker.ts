self.onmessage = async (e) => {
  const window = globalThis;
  window.window = globalThis as any;
  if (!e.data) return;
  if (typeof e.data !== 'object') return;
  if (e.data.to !== 'pokerWasm') return;
  // await fetchWasm()
  const pokerWasm = await import('src/wasm/poker_wasm');
  const { requestId, data, action, to } = e.data;
  const end = (response: any) => {
    self.postMessage({ requestId, from: to, data: response });
  };
};

const fetchWasm = async (url: string) => {
  const xhr = new XMLHttpRequest();
  xhr.open('GET', url, true);
  const res = await new Promise<any>((resolve) => {
    try {
      xhr.onload = () => {
        if (xhr.status !== 200) {
          resolve(fetchWasm(url));
          return;
        }
        resolve(xhr.response);
      };
      xhr.send();
      xhr.onerror = () => {
        resolve(fetchWasm(url));
      };
    } catch (err: any) {
      resolve(fetchWasm(url));
    }
  });
  return res;
};

export {};
