import { atom, useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useAccount, useReadContracts } from 'wagmi';
import { zeroAddress } from 'viem';
import { stateAppWallet } from 'src/states/wallet';
import { ReadContractsData } from 'wagmi/query';
import { useEffect, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { pokerApiAbi, simpleGameAbi } from 'src/wagmi.abi';

export enum RoomStatus {
  // the room not exist
  None,
  // room is opening for all players
  Opening,
  // waiting sequencer accept the offer
  Waiting,
  // room is playing
  Playing,
  // the room is over
  Over,
}

export type PageGameReades = ReadContractsData<[{ address: '0x'; abi: typeof simpleGameAbi; functionName: 'rooms' }], false>;

export interface CardReveal {
  0: `0x${string}`[];
  1: `0x${string}`;
  2: `0x${string}`;
}
export type CardReveal16 = CardReveal[];
export interface PokerCard {
  value: number;
  suit: number;
  key: number;
  data?: { value: `0x${string}`[]; reveals: CardReveal[] };
}

export const PokerValues: PokerCard[] = Array.from({ length: 48 }).map((_, key) => {
  let value = Math.floor(key / 4);
  let suit = (key % 4) + 1;
  // [♣3,♦3,♥3,♠3,♣4,♦4,♥4,♠4,♣5,♦5,♥5,♠5,♣6,♦6,♥6,♠6,♣7,♦7,♥7,♠7,♣8,♦8,♥8,♠8,♣9,♦9,♥9,♠9,♣10,♦10,♥10,♠10,♣J,♦J,♥J,♠J,♣Q,♦Q,♥Q,♠Q,♣K,♦K,♥K,♠K,♦A,♥A,♠A,♥2]
  // ♦A,♥A,♠A,♥2
  if (key === 44) {
    value = 20;
    suit = 3;
    key = 47;
  } else if (key > 44) {
    key--;
  }
  return { value, suit, key };
});
export interface PokerPlayLog {
  player: `0x${string}`;
  action: string;
  round?: number;
  turn?: number;
  cards?: PokerCard[];
  id: string;
}

export interface PokerPlayLogRound {
  logs: PokerPlayLog[];
  name: string;
}

export const PageGameWantsState = atom({ key: 'PageGameWantsState', default: [] as number[] });

export const PageGameState = atom({
  key: 'PageGameState',
  default: {
    roomId: 0n,
    room: {
      viewable: true,
      ticket: 0n,
      reward: 0n,
      sequencer: zeroAddress as `0x${string}`,
      locked: 0n,
      site: 0n,
      result: zeroAddress as `0x${string}`,
      status: RoomStatus.None,
      websocket: '',
      staking: 0n,
    },
    players: [] as `0x${string}`[],
    logs: [] as PokerPlayLogRound[],
  },
});

export const PageGameDataState = atom({
  key: 'PageGameDataState',
  default: {
    playerCards: {} as Record<string, `0x${string}`[][]>,
    playerOrder: [] as `0x${string}`[],
    round_id: 0,
    turn_id: 0,
    reveals: {} as Record<string, Array<[CardReveal, CardReveal]>>,
    myCards: Array.from({ length: 16 }).map((_, key) => ({ value: 0, suit: 0, key: key + 10000 })) as Array<PokerCard>,
    consumeCards: {} as Record<string, PokerCard[]>,
    consumeTurn: 0,
    consumeRound: 0,
    consumeLogs: [] as PokerPlayLog[],
    logs: [] as PokerPlayLogRound[],
    alias: {} as Record<string, `0x${string}`>,
  },
});

export const usePageGameState = () => {
  const param = useParams();
  const roomId = useMemo(() => BigInt(param.roomId || '0'), [param.roomId]);
  const { chain, config } = useRecoilValue(stateAppWallet);
  const _pageState = useSetRecoilState(PageGameState);
  const infos = useReadContracts({
    contracts: [
      { chainId: chain.id, address: config.SimpleGame, abi: simpleGameAbi, functionName: 'rooms', args: [roomId] },
      // { chainId: chain.id, address: config.SimpleGame, abi: simpleGameAbi, functionName: 'roomInfo', args: [roomId] },
      { chainId: chain.id, address: config.PokerApi, abi: pokerApiAbi, functionName: 'getRoomInfo', args: [config.SimpleGame, roomId] },
    ],
    batchSize: 0,
    allowFailure: false,
    query: {
      refetchInterval: 5000,
      select(data) {
        console.log('usePageGameState', roomId, data);
        const roomInfo = data[0];
        const begin: PokerPlayLog[] = [];
        const logs: PokerPlayLogRound[] = [{ logs: begin, name: 'Prepare' }];
        const players = [...data[1][1]];
        players.forEach((player) => {
          begin.unshift({ player, id: `${player}-join`, action: 'Join' });
        });
        const room = {
          viewable: roomInfo[0],
          ticket: roomInfo[1],
          reward: roomInfo[2],
          sequencer: roomInfo[3],
          locked: roomInfo[4],
          site: roomInfo[5],
          result: roomInfo[6],
          status: roomInfo[7],
          websocket: '',
          staking: 0n,
        };
        if (room.status > RoomStatus.Opening) {
          begin.unshift({ id: 'open', action: 'StartGame', player: '0x' });
        }
        if (room.sequencer !== zeroAddress) {
          const sequencerInfo = data[1][2];
          begin.unshift({ player: room.sequencer, id: 'accept', action: 'GameNode' });
          room.websocket = sequencerInfo.websocket;
          room.staking = sequencerInfo.staking;
        }
        return { roomId, room, players, logs };
      },
    },
  });

  useEffect(() => {
    if (!infos.data) return;
    console.log(infos.data);
    _pageState(infos.data);
  }, [infos.data]);

  return infos;
};
