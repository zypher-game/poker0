import { useEffect, useState } from 'react';

export function usePromise<T>(req: Promise<T>): T | null {
  const [res, _res] = useState<T | null>(null);
  useEffect(() => {
    req
      .then((result) => {
        _res(result);
      })
      .catch((err) => {
        console.error('usePromise', err);
        _res(null);
      });
  }, [req]);
  return res;
}
