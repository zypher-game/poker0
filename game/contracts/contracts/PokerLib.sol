// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;
import './RoomMarket.sol';

library PokerLib {
  struct RoomInfo {
    address sequencer;
    uint256 site;
    RoomStatus status;
  }
}
