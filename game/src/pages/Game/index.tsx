import React, { useEffect, useMemo, useRef, useState } from 'react';
import styled from 'styled-components';
import { PokerAvatar } from '../../components/PokerAvatar';
import { PokerCardCpt } from '../../components/PokerCard';
import { CardReveal, PageGameDataState, PageGameState, PageGameWantsState, PokerCard, PokerPlayLog, PokerValues, RoomStatus, usePageGameState } from './states';
import { useRecoilState, useRecoilValue } from 'recoil';
import { useWallet } from 'src/hook/useWallet';
import { simpleGameAbi } from 'src/wagmi.abi';
import { AppButton } from 'src/components/Com/AppButton';
import { GamePagePlayer } from './GamePlayer';
import { useWss } from 'src/hook/useWss';
import { pokerWasm } from 'src/lib/poker-wasm';
import { usePromise } from 'src/hook/usePromise';
import { GlobalVar } from 'src/constants';
import { Poker0Game } from 'src/types/poker';
import { stateUserZkss } from 'src/states/wallet';
import { sleep } from 'src/utils';
import { RowPoker0Ai } from 'src/lib/RowPoker0AI';

const PRC_ID = {
  FETCH_DATA_CONN: 100,
};

let nextConnTime = 0;

export const GamePage: React.FC<{}> = (props) => {
  const infos = usePageGameState();
  const [wantShows, _wantShows] = useRecoilState(PageGameWantsState);
  const gameState = useRecoilValue(PageGameState);
  const zkss = useRecoilValue(stateUserZkss);
  const zks = useMemo(() => zkss[String(gameState.roomId)], [zkss, gameState.roomId]);
  const [gameData, _gameData] = useRecoilState(PageGameDataState);
  const pw = usePromise(pokerWasm.mounted);
  const wallet = useWallet();
  const gameWss = useWss(gameState.room.http, [gameState.roomId], async (msg) => {
    const gid = Number(gameState.roomId);
    if (msg.error) {
      GlobalVar.notification.error({ message: `Code: ${msg.error.code}`, description: msg.error.message });
    }
    if (msg.method === 'online') {
      if (!zks) return;
      _gameData((v) => {
        const reveals = { ...v.reveals };
        const rels = msg.result[2] as Record<string, Array<[CardReveal, CardReveal]>>;
        const playerOrder = msg.result[1].player_order as `0x${string}`[];
        const playerCards: Record<string, `0x${string}`[][]> = {};
        const alias: Record<string, `0x${string}`> = {};
        playerOrder.forEach((peer, index) => {
          reveals[peer] = rels[peer];
          playerCards[peer] = msg.result[0][peer];
          alias[peer] = gameState.players[index];
        });
        return Poker0Game.fmtRound({
          ...v,
          playerCards,
          round_id: msg.result[1].round_id,
          turn_id: msg.result[1].turn_id,
          playerOrder: msg.result[1].player_order,
          alias,
          reveals,
          consumeLogs: (msg.result[3] as PokerPlayLog[]).map((log, i) => {
            log.id = i.toString();
            log.cards = log.cards?.map((c) => PokerValues[c as any]);
            return log;
          }),
        });
      });
      const unMaskPlayer = Object.entries(msg.result[2]).find(([peer, value]) => {
        if (peer === zks.peer) return false;
        return (value as any[]).find((c) => !c || c.length < 2) !== undefined;
      });
      // 发现其他两人的牌还未被当前用户揭秘, 发送揭秘请求
      if (!unMaskPlayer) return;
      const wasm = await pokerWasm.mounted;
      const params: any[] = [[], []];
      Object.keys(msg.result[0]).forEach((peer) => {
        if (peer === zks.peer) return;
        params[0].push(peer);
        params[1].push(wasm.batch_reveal_card(zks.sk, msg.result[0][peer]));
      });
      gameWss.send({ peer: zks.peer, gid, method: 'revealResponse', params });
      return;
    }
    if (msg.method === 'revealResponse' && msg.id !== PRC_ID.FETCH_DATA_CONN) {
      if (!zks) return;
      const diff = Math.max(0, nextConnTime - Date.now());
      await sleep(diff);
      gameWss.send({ peer: zks.peer, id: PRC_ID.FETCH_DATA_CONN, gid, method: 'connect' });
      nextConnTime = Date.now() + 10000;
      return;
    }
    if (msg.method === 'play') {
      _gameData((v) =>
        Poker0Game.fmtRound({
          ...v,
          consumeLogs: [...v.consumeLogs, { id: String(v.consumeLogs.length), action: 'play', cards: msg.result[1]?.map((c: number) => PokerValues[c]), player: msg.result[0] }],
        }),
      );
      return;
    }
    if (msg.method === 'pass') {
      _gameData((v) =>
        Poker0Game.fmtRound({
          ...v,
          consumeLogs: [...v.consumeLogs, { id: String(v.consumeLogs.length), action: 'pass', player: msg.result[0] }],
        }),
      );
      return;
    }
  });

  useEffect(() => {
    if (!zks || !pw) return;
    const revls = gameData.reveals[zks.peer];
    if (!revls?.[0]?.length || revls[0].length < 2) return;
    const cards = gameData.playerCards[zks.peer];
    if (!cards) return;
    if (gameData.myCards.length && !gameData.myCards.find((c) => c.suit === 0)) return;
    const revealsList: any[][] = cards.map(() => []);
    const revealsTemp: any[] = cards.map(() => []);
    revls.forEach((card, index) => {
      if (!cards[index]) return;
      revealsList[index].push(card[0][0]);
      revealsList[index].push(card[1][0]);
      const res = pw.reveal_card(zks.sk, cards[index]);
      revealsList[index].push(res.card);
      revealsTemp[index] = [{ card: card[0][0], proof: card[0][1], public_key: card[0][2] }, { card: card[1][0], proof: card[1][1], public_key: card[1][2] }, res];
    });
    const indexs: number[] = pw.batch_unmask_card(cards, revealsList);
    console.log('indexs', indexs);
    const myCards = indexs.map((c: number) => ({ ...PokerValues[c] }));
    myCards.forEach((card, index) => {
      card.data = { value: cards[index], reveals: revealsTemp[index] };
    });
    myCards.sort((a, b) => a.key - b.key);
    _gameData({ ...gameData, myCards });
  }, [gameData, zks, pw]);

  useEffect(() => {
    if (!zks) return;
    if (!gameState.roomId) return;
    gameWss.send({ peer: zks.peer, gid: Number(gameState.roomId), method: 'connect' });
    let live = true;
    const timer = setInterval(() => {
      if (!live) return;
      gameWss.send({ peer: zks.peer, gid: Number(gameState.roomId), method: 'connect' });
    }, 10000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [gameWss, zks]);

  const users = useMemo(() => {
    let index = -1;
    index = gameState.players.indexOf(wallet.address);
    if (index === -1) index = 0;
    const nextIndex = (index + 1) % 3;
    const nextIndex2 = (index + 2) % 3;
    const pokerFilter = (key: number) => {
      const peer = gameData.playerOrder[key];
      const consumeCards = gameData.consumeCards[peer];
      const address = gameState.players[key];
      let pokers: PokerCard[] = Array.from({ length: 16 - (consumeCards?.length ?? 0) }).map((_, i) => ({ value: 0, suit: 0, key: (key + 1) * 1000 + i }));
      if (address === wallet.address && gameData.myCards.length) {
        pokers = gameData.myCards.filter((c) => {
          if (!consumeCards) return true;
          const consumed = gameData.consumeCards[peer].find((card) => card.key === c.key);
          if (consumed) return false;
          return true;
        });
      }
      return { key, address, pokers };
    };
    return [pokerFilter(nextIndex), pokerFilter(nextIndex2), pokerFilter(index)];
  }, [wallet.address, gameState, gameData]);

  const activeIndex = useMemo(() => {
    // 游戏还没开始, 或者红桃叁还没出
    if (gameData.consumeLogs.length === 0) {
      if (users[2].pokers.find((c) => c.key === 2)) {
        console.log('find 33333');
        return users[2].key;
      }
      return -1;
    }
    const lastAction = gameData.consumeLogs[gameData.consumeLogs.length - 1];
    return (gameData.playerOrder.indexOf(lastAction.player) + 1) % 3;
  }, [gameState, users, wallet.address]);

  useEffect(() => {
    if (activeIndex !== users[2].key) return;
    let lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 1];
    if (!lastLog) return;
    if (['pass'].includes(lastLog.action)) lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 2];
    if (!lastLog || ['pass'].includes(lastLog.action) || !lastLog.cards) return; // 自由出牌阶段
    const myCards = gameData.myCards;
    const publicCards: PokerCard[] = [];
    const limit = Poker0Game.getCardsTypeAndValue([...lastLog.cards]);
    if (!limit) return;
    Object.values(gameData.consumeCards).forEach((v) => publicCards.push(...v));
    const cards = RowPoker0Ai(publicCards, myCards, limit);
    if (!cards) return;
    _wantShows(cards.map((c) => c.key));
  }, [activeIndex, users[2]?.key, gameData.myCards]);

  // console.log(activeIndex, zks);

  return (
    <GamePageIndexStyle>
      {users.map((user) => (
        <GamePagePlayer gameState={gameState} wallet={wallet} key={user.key} className="user-info" active={activeIndex === user.key} user={user} />
      ))}
      {/* player start game */}
      {gameState.room.status === RoomStatus.Opening && (
        <div className="actions">
          <AppButton
            disabled={wallet.address !== gameState.players[0]}
            onClick={wallet.handler({
              action: 'StartGame',
              callback: async () => {
                return wallet.write.writeContractAsync({
                  abi: simpleGameAbi,
                  address: wallet.appWallet.config.SimpleGame,
                  functionName: 'startRoom',
                  args: [gameState.roomId],
                });
              },
              onSuccess: () => infos.refetch(),
            })}
          >
            StartGame
          </AppButton>
        </div>
      )}
      {gameState.room.status > RoomStatus.Opening && (
        <div className="actions">
          {wantShows.length > 0 && (
            <AppButton className="cancel" onClick={() => _wantShows([])}>
              Cancel
            </AppButton>
          )}
          <AppButton
            className="cancel"
            onClick={async () => {
              if (!zks) return;
              if (activeIndex !== users[2].key) return;
              let lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 1];
              if (!lastLog) return GlobalVar.message.error('Invalid pass1');
              if (['paas', 'pass'].includes(lastLog.action)) lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 2];
              console.log('lastLog', lastLog, gameData.consumeLogs);
              if (!lastLog || ['paas', 'pass'].includes(lastLog.action) || !lastLog.cards) return GlobalVar.message.error('Invalid pass2');
              const ct = Poker0Game.getCardsTypeAndValue([...lastLog.cards]);
              if (!ct) return GlobalVar.message.error('Invalid Cards');
              const pw = await pokerWasm.mounted;
              const arg = {
                round_id: gameData.consumeRound,
                turn_id: gameData.consumeTurn,
                room_id: Number(gameState.roomId),
                action: 1,
                types: ct.type,
                play_cards: [],
                reveals: [],
                private_key: zks.sk,
              };
              const env = pw.create_play_env(arg);
              await gameWss.request({ peer: zks.peer, gid: Number(gameState.roomId), method: 'pass', params: [env] });
            }}
          >
            Pass
          </AppButton>
          <AppButton
            className="ok"
            onClick={async () => {
              if (!zks) return;
              if (activeIndex !== users[2].key) return;
              const limit = (() => {
                let lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 1];
                if (!lastLog) return null;
                if ('pass' === lastLog.action) lastLog = gameData.consumeLogs[gameData.consumeLogs.length - 2];
                if (!lastLog || !lastLog.cards || lastLog.action !== 'play') return null;
                return Poker0Game.getCardsTypeAndValue([...lastLog.cards]);
              })();

              const pokers = users[2].pokers;
              const cards = pokers.filter((c) => wantShows?.includes(c.key));
              const ct = Poker0Game.getCardsTypeAndValue(cards);
              if (!ct) return GlobalVar.message.error('Invalid Cards');
              if (limit && ct.type !== limit.type) return GlobalVar.message.error('Invalid Card Type');
              if (limit && ct.value <= limit.value) return GlobalVar.message.error('Invalid Card Value');
              const pw = await pokerWasm.mounted;
              const arg = {
                round_id: gameData.consumeRound,
                turn_id: gameData.consumeTurn,
                room_id: Number(gameState.roomId),
                action: 0,
                types: ct.type,
                play_cards: ct.cards.map((c) => c.data?.value),
                reveals: ct.cards.map((c) => c.data?.reveals),
                private_key: zks.sk,
              };
              const env = pw.create_play_env(arg);
              await gameWss.request({ peer: zks.peer, gid: Number(gameState.roomId), method: 'play', params: [env] });
            }}
          >
            Ok
          </AppButton>
        </div>
      )}
      <div className="com">
        <div className="rounds">
          {gameData.logs.concat(gameState.logs).map((round) => (
            <div key={round.name} className="round">
              {round.logs.map((log, j) => (
                <div key={j} className="times">
                  {log.player !== '0x' && <PokerAvatar address={gameData.alias[log.player] || log.player} size={24} />}
                  {log.cards && (
                    <div className="cards">
                      {log.cards.map((card) => (
                        <PokerCardCpt className="card" smini suit={card.suit} value={card.value} key={card.key} />
                      ))}
                    </div>
                  )}
                  {!log.cards && log.action && <div className="pass">{log.action}</div>}
                  {/* <div className="pass">Pass</div> */}
                </div>
              ))}
              <div className="round-index">Round: {round.name}</div>
            </div>
          ))}
        </div>
      </div>
    </GamePageIndexStyle>
  );
};

const GamePageIndexStyle = styled.div`
  width: 100%;
  display: flex;
  flex-wrap: wrap;
  justify-content: space-around;
  align-items: center;
  position: relative;
  gap: 200px 10px;
  padding-top: 100px;
  padding-bottom: 40px;
  > .com {
    position: absolute;
    left: 0;
    right: 0;
    top: 180px;
    margin: auto;
    width: 30vw;
    height: 230px;
    overflow: auto;
    display: flex;
    flex-direction: column-reverse;
    background-color: #1b4773;
    box-shadow: 0 0 20px 4px #1b4773;
    padding: 10px;
    border-radius: 8px;
    > .rounds {
      width: 30vw;
      display: flex;
      flex-direction: column-reverse;
      gap: 10px;
      > .round {
        width: 30vw;
        display: flex;
        flex-direction: column-reverse;
        gap: 10px;
        > .round-index {
          font-size: 12px;
          color: #fff;
          text-align: left;
        }
        > .times {
          display: flex;
          gap: 6px;
          justify-content: space-between;
          align-items: center;
          > .pass {
            color: #fff;
            font-size: 20px;
          }
          > .cards {
            display: flex;
            > .card + .card {
              margin-left: -8px;
            }
          }
        }
      }
    }
  }
  > .user-info {
    &:nth-child(1) {
      transform: rotateZ(-45deg);
      > .cards {
        > .card + .card {
          margin-left: -20px;
        }
      }
    }
    &:nth-child(2) {
      transform: rotateZ(45deg);
      > .cards {
        > .card + .card {
          margin-left: -20px;
        }
      }
    }
    &:nth-child(3) {
      > .avatar {
        > span {
          transform: rotateZ(45deg);
        }
      }
    }
  }
  > .actions {
    position: absolute;
    width: 40vw;
    bottom: 10px;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: space-around;
    > div {
      padding: 4px 8px;
    }
  }
`;
