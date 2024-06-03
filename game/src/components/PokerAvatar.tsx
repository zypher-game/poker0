import { AvatarComponent } from '@rainbow-me/rainbowkit';
import { Collapse, CollapseProps } from 'antd';
import React, { useEffect, useMemo, useState } from 'react';
import { Outlet, useMatch } from 'react-router-dom';
import styled from 'styled-components';
import { Address, hexToBigInt, zeroAddress } from 'viem';
import classNames from 'classnames';
import { PokerSuitValue, PokerValueMap } from '../types/poker';

const genAvatar = (address: Address, size: number) => {
  const value = hexToBigInt(address);
  const suitValue = 1 + Number(value % 4n);
  const suit = PokerSuitValue[suitValue as 1];
  let valIndex = Number((value / 4n) % 13n);
  if (valIndex === 12) valIndex = 20;
  const val = PokerValueMap[valIndex];
  const style: React.CSSProperties = useMemo(() => {
    const tsv = hex2int(address[25]);
    return {
      backgroundImage: `linear-gradient(${hex2int(address.slice(2, 4)) % 180}deg, #${address.slice(4, 7)} 0%, #${address.slice(7, 10)} 50%, #${address.slice(10, 13)} 100%)`,
      backgroundColor: `#${address.slice(13, 16)}`,
      width: `${size}px`,
      height: `${size}px`,
      boxShadow: `0px 0px 0px 1px #${address.slice(16, 19)}`,
      color: `#${address.slice(19, 22)}`,
      textShadow: `${(tsv % 3) - 1}px ${(Math.floor(tsv / 3) % 3) - 1}px 1px #${address.slice(21, 25)}`,
      borderRadius: (hex2int(address[26]) % 8) + 4,
    };
  }, [address]);
  return { value, suit, suitValue, val, style };
};
const hex2int = (hex: string) => parseInt(hex, 16);

export const PokerAvatar: AvatarComponent = (props) => {
  if (props.ensImage) {
    return <img title={props.address} src={props.ensImage} width={props.size} height={props.size} style={{ borderRadius: 999 }} />;
  }
  const avatar = genAvatar((props.address as Address) || zeroAddress, props.size);
  return (
    <PageStyle title={props.address} style={avatar.style} className={classNames(`poker-avatar suit-${avatar.suitValue}`)}>
      <div className="suit">{avatar.suit}</div>
      <div className="val">{avatar.val}</div>
    </PageStyle>
  );
};

const PageStyle = styled.div`
  width: 24px;
  height: 24px;
  position: relative;
  font-size: 12px;
  /* border-radius: 6px; */
  user-select: none;
  &.suit-1,
  &.suit-4 {
    /* color: #000; */
    /* -webkit-text-stroke: 1px #fff; */
  }
  &.suit-2,
  &.suit-3 {
    /* color: #750024; */
    /* -webkit-text-stroke: 1px #fff; */
  }

  > .suit {
    position: absolute;
    bottom: 2px;
    right: 2px;
    /* transform: scale(0.9); */
  }
  > .val {
    position: absolute;
    top: 2px;
    left: 2px;
    /* transform: scale(0.9); */
  }
`;
