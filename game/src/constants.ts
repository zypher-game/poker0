import { MessageInstance } from 'antd/es/message/interface';
import { NotificationInstance } from 'antd/es/notification/interface';
import { zeroAddress } from 'viem';

export const AppConstants = {
  // WSS: 'http://localhost:10026',
  WSS: 'https://poker-chat.zypher.game',
};

export const ethAddress = '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE' as const;

export const roomPlayerNum = 3;

export const GlobalVar = {
  notification: null as any as NotificationInstance,
  message: null as any as MessageInstance,
};
