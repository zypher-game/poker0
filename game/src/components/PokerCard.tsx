import React, { HTMLProps, useCallback, useContext, useEffect, useState } from 'react';
import styled from 'styled-components';
import classNames from 'classnames';
import { ReactComponent as PokerBg } from '../assets/svg/poker-bg.svg';
import { PokerSuit, PokerSuitValue, PokerValueMap } from '../types/poker';

export const PokerCardCpt: React.FC<{
  onClick?: () => any;
  className?: string;
  style?: React.CSSProperties;
  suit: PokerSuit;
  value: number;
  isNull?: boolean;
  mini?: boolean;
  smini?: boolean;
  active?: boolean;
  // eslint-disable-next-line react/display-name
}> = React.memo((props) => {
  return (
    <CptStyle
      onClick={props.onClick}
      className={classNames(props.className, { active: props.active, isNull: props.isNull, mini: props.mini, smini: props.smini }, `value-${props.value}`, `type-${props.suit}`)}
      style={props.style}
    >
      <div className="front">
        <div className="value">{PokerValueMap[props.value]}</div>
        <div className="type">{PokerSuitValue[props.suit]}</div>
      </div>
      <div className="back">
        <PokerBg />
      </div>
    </CptStyle>
  );
});
const CptStyle = styled.div`
  width: 60px;
  height: 76px;
  box-sizing: border-box;
  border-radius: 4px;
  font-size: 26px;
  font-weight: 800;
  box-sizing: border-box;
  background-color: #fff;
  line-height: 1;
  position: relative;
  font-family: Saira;
  cursor: pointer;
  transition: all 0.5s ease-in-out;
  backface-visibility: hidden;
  transform-style: preserve-3d;
  transform: rotateY(180deg);
  user-select: none;
  box-shadow: black -1px 1px 3px 0px;
  &:hover,
  &.active {
    background-color: #abece1;
  }
  &.mini {
    width: 40px;
    height: 52px;
    font-size: 16px;
  }
  &.smini {
    width: 28px;
    height: 36px;
    font-size: 12px;
    border-radius: 3px;
  }
  /* &.active {
    > .front {
      background: linear-gradient(145deg, rgb(242, 242, 242) 0%, rgb(0 174 255 / 59%) 100%);
    }
  } */

  // 未知的牌
  &.type-0 {
    &.isNull {
      transform: rotateY(0deg);
      background-color: transparent;
      border: 1px solid rgba(255, 255, 255, 0.1);
      box-shadow: none;
      > .back > svg {
        display: none;
      }
    }
  }
  &.type-1,
  &.type-2,
  &.type-3,
  &.type-4 {
    transform: rotateY(0deg);
  }

  > .back {
    backface-visibility: hidden;
    transform: rotateY(180deg);
    position: absolute;
    top: 0;
    left: 0;
    z-index: 1;
    border-radius: 4px;
    box-shadow: black 0px 0px 3px 0px;
    overflow: hidden;
    > svg {
      width: 100%;
      height: 100%;
    }
  }
  > .front {
    backface-visibility: hidden;
    height: 100%;
    border-radius: 4px;
    > .value {
      padding-left: 4px;
      padding-top: 2px;
    }
    > .type {
      position: absolute;
      bottom: 4px;
      left: 4px;
    }
  }
  &.showCard-enter {
    transform: rotateY(0);
  }
  &.showCard-enter-active {
    transform: rotateY(180deg);
    transition: all 0.5s;
  }
  &.fetch-card-enter {
    transform: rotateY(0);
  }
  &.fetch-card-enter-active {
    transform: rotateY(180deg);
    transition: all 0.5s;
  }
  /* Spade, // .......... 1 (♠)
  Heart, // .......... 2 (♥)
  Diamond, // ........ 3 (♦)
  Club, // ............ 4 (♣) */
  &.type-2,
  &.type-3 {
    color: #dd2e44;
  }
  &.type-1,
  &.type-4 {
    color: #2f2f2f;
  }
`;
