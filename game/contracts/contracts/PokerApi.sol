// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;
import './RoomMarket.sol';

contract PokerApi {
  struct RoomInfo {
    address sequencer;
    uint256 site;
    RoomStatus status;
  }
  function getRoomInfo(RoomMarket market, uint256 roomId) public view returns (RoomInfo memory room, address[] memory players, RoomMarket.Sequencer memory sequencer) {
    (players, , room.sequencer, room.site, room.status) = market.roomInfo(roomId);
    (sequencer.http, sequencer.staking) = market.sequencers(room.sequencer);
  }
}
