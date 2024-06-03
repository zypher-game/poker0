//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// PokerApi
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const pokerApiAbi = [
  {
    type: 'function',
    inputs: [
      { name: 'market', internalType: 'contract RoomMarket', type: 'address' },
      { name: 'roomId', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'getRoomInfo',
    outputs: [
      {
        name: 'room',
        internalType: 'struct PokerApi.RoomInfo',
        type: 'tuple',
        components: [
          { name: 'sequencer', internalType: 'address', type: 'address' },
          { name: 'site', internalType: 'uint256', type: 'uint256' },
          { name: 'status', internalType: 'enum RoomStatus', type: 'uint8' },
        ],
      },
      { name: 'players', internalType: 'address[]', type: 'address[]' },
      {
        name: 'sequencer',
        internalType: 'struct RoomMarket.Sequencer',
        type: 'tuple',
        components: [
          { name: 'http', internalType: 'string', type: 'string' },
          { name: 'staking', internalType: 'uint256', type: 'uint256' },
        ],
      },
    ],
    stateMutability: 'view',
  },
] as const;

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// SimpleGame
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const simpleGameAbi = [
  {
    type: 'constructor',
    inputs: [
      { name: '_token', internalType: 'address', type: 'address' },
      { name: '_minStaking', internalType: 'uint256', type: 'uint256' },
      { name: '_playerRoomLock', internalType: 'uint256', type: 'uint256' },
      { name: '_playerLimit', internalType: 'uint256', type: 'uint256' },
      { name: '_startRoomId', internalType: 'uint256', type: 'uint256' },
    ],
    stateMutability: 'nonpayable',
  },
  { type: 'error', inputs: [{ name: 'owner', internalType: 'address', type: 'address' }], name: 'OwnableInvalidOwner' },
  { type: 'error', inputs: [{ name: 'account', internalType: 'address', type: 'address' }], name: 'OwnableUnauthorizedAccount' },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'room', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'sequencer', internalType: 'address', type: 'address', indexed: false },
      { name: 'http', internalType: 'string', type: 'string', indexed: false },
      { name: 'locked', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'params', internalType: 'bytes', type: 'bytes', indexed: false },
    ],
    name: 'AcceptRoom',
  },
  { type: 'event', anonymous: false, inputs: [{ name: 'room', internalType: 'uint256', type: 'uint256', indexed: false }], name: 'ClaimRoom' },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'room', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'game', internalType: 'address', type: 'address', indexed: false },
      { name: 'reward', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'player', internalType: 'address', type: 'address', indexed: false },
      { name: 'peer', internalType: 'address', type: 'address', indexed: false },
      { name: 'pk', internalType: 'bytes32', type: 'bytes32', indexed: false },
    ],
    name: 'CreateRoom',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'room', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'player', internalType: 'address', type: 'address', indexed: false },
      { name: 'peer', internalType: 'address', type: 'address', indexed: false },
      { name: 'pk', internalType: 'bytes32', type: 'bytes32', indexed: false },
    ],
    name: 'JoinRoom',
  },
  { type: 'event', anonymous: false, inputs: [{ name: 'room', internalType: 'uint256', type: 'uint256', indexed: false }], name: 'OverRoom' },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'previousOwner', internalType: 'address', type: 'address', indexed: true },
      { name: 'newOwner', internalType: 'address', type: 'address', indexed: true },
    ],
    name: 'OwnershipTransferred',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: '', internalType: 'address', type: 'address', indexed: false },
      { name: 'win', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'reward', internalType: 'uint256', type: 'uint256', indexed: false },
    ],
    name: 'Ranking',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'sequencer', internalType: 'address', type: 'address', indexed: false },
      { name: 'http', internalType: 'string', type: 'string', indexed: false },
      { name: 'staking', internalType: 'uint256', type: 'uint256', indexed: false },
    ],
    name: 'StakeSequencer',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'room', internalType: 'uint256', type: 'uint256', indexed: false },
      { name: 'game', internalType: 'address', type: 'address', indexed: false },
    ],
    name: 'StartRoom',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'sequencer', internalType: 'address', type: 'address', indexed: false },
      { name: 'staking', internalType: 'uint256', type: 'uint256', indexed: false },
    ],
    name: 'UnstakeSequencer',
  },
  {
    type: 'function',
    inputs: [
      { name: 'roomId', internalType: 'uint256', type: 'uint256' },
      { name: 'params', internalType: 'bytes', type: 'bytes' },
    ],
    name: 'acceptRoom',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  { type: 'function', inputs: [{ name: 'roomId', internalType: 'uint256', type: 'uint256' }], name: 'claimRoom', outputs: [], stateMutability: 'nonpayable' },
  {
    type: 'function',
    inputs: [
      { name: 'ticket', internalType: 'uint256', type: 'uint256' },
      { name: 'peer', internalType: 'address', type: 'address' },
      { name: 'pk', internalType: 'bytes32', type: 'bytes32' },
    ],
    name: 'createRoom',
    outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'sequencer', internalType: 'address', type: 'address' }],
    name: 'isSequencer',
    outputs: [{ name: '', internalType: 'bool', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [
      { name: 'roomId', internalType: 'uint256', type: 'uint256' },
      { name: 'peer', internalType: 'address', type: 'address' },
      { name: 'pk', internalType: 'bytes32', type: 'bytes32' },
    ],
    name: 'joinRoom',
    outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }],
    stateMutability: 'nonpayable',
  },
  { type: 'function', inputs: [], name: 'minStaking', outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }], stateMutability: 'view' },
  { type: 'function', inputs: [], name: 'nextRoomId', outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }], stateMutability: 'view' },
  {
    type: 'function',
    inputs: [
      { name: 'roomId', internalType: 'uint256', type: 'uint256' },
      { name: 'data', internalType: 'bytes', type: 'bytes' },
      { name: 'proof', internalType: 'bytes', type: 'bytes' },
    ],
    name: 'overRoomWithThreshold',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'roomId', internalType: 'uint256', type: 'uint256' },
      { name: 'data', internalType: 'bytes', type: 'bytes' },
      { name: 'proof', internalType: 'bytes', type: 'bytes' },
    ],
    name: 'overRoomWithZk',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  { type: 'function', inputs: [], name: 'owner', outputs: [{ name: '', internalType: 'address', type: 'address' }], stateMutability: 'view' },
  { type: 'function', inputs: [], name: 'playerLimit', outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }], stateMutability: 'view' },
  { type: 'function', inputs: [], name: 'playerRoomLock', outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }], stateMutability: 'view' },
  { type: 'function', inputs: [], name: 'renounceOwnership', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [{ name: 'roomId', internalType: 'uint256', type: 'uint256' }], name: 'restartRoom', outputs: [], stateMutability: 'nonpayable' },
  {
    type: 'function',
    inputs: [{ name: 'roomId', internalType: 'uint256', type: 'uint256' }],
    name: 'roomInfo',
    outputs: [
      { name: '', internalType: 'address[]', type: 'address[]' },
      { name: '', internalType: 'address', type: 'address' },
      { name: '', internalType: 'address', type: 'address' },
      { name: '', internalType: 'uint256', type: 'uint256' },
      { name: '', internalType: 'enum RoomStatus', type: 'uint8' },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [{ name: '', internalType: 'uint256', type: 'uint256' }],
    name: 'rooms',
    outputs: [
      { name: 'ticket', internalType: 'uint256', type: 'uint256' },
      { name: 'reward', internalType: 'uint256', type: 'uint256' },
      { name: 'sequencer', internalType: 'address', type: 'address' },
      { name: 'locked', internalType: 'uint256', type: 'uint256' },
      { name: 'site', internalType: 'uint256', type: 'uint256' },
      { name: 'result', internalType: 'bytes', type: 'bytes' },
      { name: 'status', internalType: 'enum RoomStatus', type: 'uint8' },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [{ name: '', internalType: 'address', type: 'address' }],
    name: 'sequencers',
    outputs: [
      { name: 'http', internalType: 'string', type: 'string' },
      { name: 'staking', internalType: 'uint256', type: 'uint256' },
    ],
    stateMutability: 'view',
  },
  { type: 'function', inputs: [{ name: '_minStaking', internalType: 'uint256', type: 'uint256' }], name: 'setMinStaking', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [{ name: '_playerLimit', internalType: 'uint256', type: 'uint256' }], name: 'setPlayerLimit', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [{ name: '_playerRoomLock', internalType: 'uint256', type: 'uint256' }], name: 'setPlayerRoomLock', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [{ name: '_token', internalType: 'address', type: 'address' }], name: 'setToken', outputs: [], stateMutability: 'nonpayable' },
  {
    type: 'function',
    inputs: [
      { name: 'http', internalType: 'string', type: 'string' },
      { name: 'amount', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'stakeSequencer',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  { type: 'function', inputs: [{ name: 'roomId', internalType: 'uint256', type: 'uint256' }], name: 'startRoom', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [], name: 'token', outputs: [{ name: '', internalType: 'address', type: 'address' }], stateMutability: 'view' },
  { type: 'function', inputs: [{ name: 'newOwner', internalType: 'address', type: 'address' }], name: 'transferOwnership', outputs: [], stateMutability: 'nonpayable' },
  { type: 'function', inputs: [{ name: 'amount', internalType: 'uint256', type: 'uint256' }], name: 'unstakeSequencer', outputs: [], stateMutability: 'nonpayable' },
] as const;
