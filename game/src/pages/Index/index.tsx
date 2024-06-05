import React, { useState } from 'react';
import styled from 'styled-components';
import { useNavigate } from 'react-router-dom';
import { FloatButton, Tooltip } from 'antd';
import { QuestionCircleOutlined, TeamOutlined } from '@ant-design/icons';
import { PokerRulesTip } from '../../components/PokerRules';
import { PokerAvatar } from '../../components/PokerAvatar';
import { PageIndexState, usePageIndexState } from './states';
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil';
import { useWallet } from 'src/hook/useWallet';
import { simpleGameAbi } from 'src/wagmi.abi';
import { pokerWasm } from 'src/lib/poker-wasm';
import { ethers } from 'ethers';
import { AppButton } from 'src/components/Com/AppButton';
import { stateUserZkss } from 'src/states/wallet';
import { getAddress, getContract, parseEventLogs } from 'viem';
import { stateLoginState, useLoginToken } from 'src/states/login';

export const PageIndex: React.FC<{}> = (props) => {
  const navi = useNavigate();
  const infos = usePageIndexState();
  const pageState = useRecoilValue(PageIndexState);
  const wallet = useWallet();
  const _zkss = useSetRecoilState(stateUserZkss);
  // const genKey = useLoginToken();

  return (
    <PageIndexStyle>
      <div className="logo">
        <img height={256} alt="" width={256} src="/favicon.png" />
      </div>
      <div className="btns">
        <AppButton
          className="CreateRoom"
          onClick={wallet.handler({
            action: 'CreateRoom',
            async callback() {
              const pw = await pokerWasm.mounted;
              // const salt = BigInt(wallet.address);
              // const key = await genKey?.(salt);
              // if (!key) return;
              const zkst = pw.generate_key_by_seed(wallet.address);
              zkst.peer = ethers.computeAddress(zkst.pk);
              _zkss((v) => ({ ...v, [1]: zkst }));
              // function createRoom(uint256 reward, bool viewable, address player, address peer, bytes32 pk) external returns (uint256) {
              return wallet.write.writeContractAsync({
                abi: simpleGameAbi,
                address: wallet.appWallet.config.SimpleGame,
                functionName: 'createRoom',
                args: [0n, true, zkst.peer, zkst.pk],
              });
            },
            onSuccess: (res) => {
              _zkss((v) => {
                const last = v[1];
                if (!last) return v;
                const logs = parseEventLogs({ abi: simpleGameAbi, logs: res.logs.filter((log) => getAddress(log.address) === wallet.appWallet.config.SimpleGame) });
                console.log(logs);
                let roomId = 1;
                logs.forEach((log) => {
                  if (getAddress(log.address) !== wallet.appWallet.config.SimpleGame) return;
                  if (log.eventName !== 'CreateRoom') return;
                  if (log.args.peer !== last.peer) return;
                  roomId = Number(log.args.room);
                });
                const nv = { ...v };
                delete nv[1];
                nv[roomId] = last;
                return nv;
              });
              infos.refetch();
            },
          })}
        >
          CreateRoom
        </AppButton>
        {/* <JoinRoomWithRoomId /> */}
      </div>
      <div className="rooms">
        {pageState.rooms.map((room, index) => {
          return (
            <div className="room" key={room.room} onClick={() => navi(`/game/${room.room}`)}>
              <div className="avatars">
                {room.players.map((player, i) => (
                  <PokerAvatar key={player + i} address={player} size={24} />
                ))}
              </div>
              <div className="roomId">#{room.room}</div>
              <div className="num">
                <TeamOutlined /> <span className="val">{room.players.length}/3</span>
              </div>
            </div>
          );
        })}
      </div>
      <Tooltip trigger="click" title={<PokerRulesTip />}>
        <FloatButton icon={<QuestionCircleOutlined />} type="primary" style={{ right: 24 }} />
      </Tooltip>
    </PageIndexStyle>
  );
};

const PageIndexStyle = styled.div`
  width: 100vw;
  min-height: 70vh;
  padding: 8px 10% 40px;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  flex-direction: column;
  justify-content: center;
  user-select: none;
  gap: 16px;
  > .rooms {
    display: flex;
    gap: 16px;
    margin-top: 40px;
    flex-wrap: wrap;
    justify-content: space-evenly;
    > .room {
      position: relative;
      display: flex;
      align-items: flex-end;
      justify-content: space-between;
      border: 4px solid #eeeeee33;
      width: 120px;
      height: 60px;
      background-color: #5a0758;
      border-radius: 12px;
      color: #b4882e;
      cursor: pointer;
      transition: all ease 0.2s;
      &:hover {
        opacity: 0.8;
        transform: scale(1.05);
      }
      > .num {
        font-size: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        padding-right: 4px;
      }
      > .avatars {
        position: absolute;
        left: 4px;
        justify-content: space-evenly;
        top: 6px;
        right: 4px;
        display: flex;
        gap: 4px;
      }
      > .roomId {
        padding-left: 4px;
        font-size: 12px;
      }
    }
  }
  > .btns {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    > .CreateRoom {
      user-select: none;
    }
    > div {
      width: 200px;
      font-size: 18px;
      height: 40px;
      padding: 4px;
    }
  }
  > .logo {
    width: 100vw;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    > img {
      user-select: none;
    }
  }
`;
