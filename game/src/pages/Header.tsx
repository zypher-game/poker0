import { ConnectButton } from '@rainbow-me/rainbowkit';
import React from 'react';
import { Link, NavLink } from 'react-router-dom';
import styled from 'styled-components';

export const AppHeader: React.FC<{}> = () => {
  return (
    <PageStyle>
      <div className="left">
        <NavLink to="/">
          <img alt="" height={24} width={24} src="/favicon.png" />
          Poker0
        </NavLink>
        {/* <Link to="/docs">Docs</Link> */}
      </div>
      <div className="right">
        <ConnectButton />
      </div>
    </PageStyle>
  );
};

const PageStyle = styled.div`
  padding: 10px;
  display: flex;
  justify-content: space-between;
  > .left {
    display: flex;
    gap: 16px;
    > a {
      text-decoration: none;
      font-size: 18px;
      color: aliceblue;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      &:hover {
        opacity: 0.8;
        text-decoration: underline;
      }
      &.active {
        color: #b6ddff;
        text-decoration: underline;
      }
    }
  }
`;
