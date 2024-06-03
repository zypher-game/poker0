// import { useEffect, useRef } from 'react';
// import { io, Socket } from 'socket.io-client';
// import { useAccount } from 'wagmi';
// import { AppConstants } from '../constants';

// export interface Msg {
//   address: `0x${string}`;
//   name: string;
//   msg: string;
//   time: number;
//   online: number;
// }

// interface NspC2SEvents {
//   reset: () => void;
//   createRoom: (msg: string) => void;
//   say: (msg: string) => void;
//   play: (cards: CardType[]) => void;
//   pass: () => void;
//   join: () => void;
// }

// export interface GameData {
//   players: string[];
//   pokers: Array<CardType[]>;
//   playingQueue: ShowCards[];
//   online: number;
// }
// export interface GameRoom {
//   roomId: string;
//   players: string[];
//   playing: boolean;
//   viewPlayers: number;
// }

// interface NspS2CEvents {
//   error: (data: string) => void;
//   msg: (data: Msg[]) => void;
//   gameSync: (data: GameData) => void;
//   rooms: (rooms: GameRoom[]) => void;
// }

// export const Heart3Key = 2 * 100 + 0;

// export const useGameRoomChat = (roomId: number | string, cb: (socket: Socket<NspS2CEvents, NspC2SEvents>) => Function) => {
//   const socketEl = useRef<Socket<NspS2CEvents, NspC2SEvents>>();
//   const acc = useAccount();
//   useEffect(() => {
//     const socket = io(`${AppConstants.WSS}/${roomId ? `poker0-${roomId}` : `poker0-rooms`}`, { query: { address: acc.address } });
//     socket.on('error', console.error);
//     const del = cb(socket);
//     socketEl.current = socket;
//     return () => {
//       del();
//       socket.disconnect();
//     };
//   }, [roomId, acc.address]);
//   return socketEl;
// };
