import { ArrowRightOutlined, CloseCircleFilled, CloseCircleOutlined } from '@ant-design/icons';
import { Input } from 'antd';
import classNames from 'classnames';
import React, { useEffect, useMemo, useState } from 'react';
import { Outlet, useMatch } from 'react-router-dom';
import { UnwrapRecoilValue, useRecoilState, useSetRecoilState } from 'recoil';
import { useWallet } from 'src/hook/useWallet';
import styled from 'styled-components';
import { PageGameState, PageGameWantsState } from './states';
import { PokerAvatar } from 'src/components/PokerAvatar';
import { pokerWasm } from 'src/lib/poker-wasm';
import { stateUserZkss } from 'src/states/wallet';
import { simpleGameAbi } from 'src/wagmi.abi';
import { ethers } from 'ethers';
import { GlobalVar } from 'src/constants';
import { errorParse } from 'src/utils';
import { PokerCardCpt } from 'src/components/PokerCard';

export const GamePagePlayer: React.FC<{
  className?: string;
  gameState: UnwrapRecoilValue<typeof PageGameState>;
  wallet: ReturnType<typeof useWallet>;
  active?: boolean;
  user: { address: `0x${string}`; pokers: any[] };
}> = ({ user, active, className, wallet, gameState }) => {
  const _zkss = useSetRecoilState(stateUserZkss);
  const [wantShows, _wantShows] = useRecoilState(PageGameWantsState);
  const [loading, setLoading] = useState(false);
  return (
    <PageStyle className={classNames(className, { active: active })}>
      <div className="avatar">
        {user.address ? (
          <PokerAvatar address={user.address} size={40} />
        ) : (
          <CloseCircleOutlined
            className={classNames({ loading })}
            onClick={wallet.handler({
              setLoading,
              action: 'JoinRoom',
              async callback() {
                const pw = await pokerWasm.mounted;
                const zkst = pw.generate_key();
                zkst.peer = ethers.computeAddress(zkst.pk);
                _zkss((v) => ({ ...v, [gameState.roomId.toString()]: zkst }));
                // function joinRoom(uint256 roomId, address player, address peer, bytes32 pk)
                return wallet.write.writeContractAsync({
                  abi: simpleGameAbi,
                  address: wallet.appWallet.config.SimpleGame,
                  functionName: 'joinRoom',
                  args: [gameState.roomId, zkst.peer, zkst.pk],
                });
              },
              onError: (err) => GlobalVar.notification.error({ message: errorParse(err) }),
              // onSuccess: () => infos.refetch(),
            })}
          />
        )}
      </div>
      <div className="cards">
        {user.pokers?.map((card, index) => (
          <PokerCardCpt
            onClick={() => {
              _wantShows((v) => {
                const newV = [...v];
                const index = newV.indexOf(card.key);
                if (index === -1) {
                  newV.push(card.key);
                } else {
                  newV.splice(index, 1);
                }
                return newV;
              });
            }}
            active={wantShows.includes(card.key)}
            className="card"
            smini
            suit={card.suit}
            value={card.value}
            key={card.key || index}
          />
        ))}
      </div>
    </PageStyle>
  );
};

const PageStyle = styled.div`
  width: 40vw;
  color: #fff;
  display: flex;
  align-items: center;
  flex-direction: column;
  justify-content: center;
  padding: 20px 10px;
  gap: 12px;
  &.active {
    > .avatar {
      border: 2px solid #4fda00;
      box-shadow: 0 0 10px 4px #4fda00;
      border-radius: 100px;
    }
  }
  > .cards {
    display: flex;
    height: 60px;
    > .card + .card {
      margin-left: -8px;
    }
    > .card.active {
      top: -8px;
      transition: all ease 0.2s;
    }
  }
  > .avatar {
    border: 2px solid transparent;
    > .loading {
      animation: rotate360 infinite 2s linear;
    }
    > span {
      cursor: pointer;
      font-size: 40px;
    }
  }
`;
