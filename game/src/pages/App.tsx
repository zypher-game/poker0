import React from 'react';
import { Outlet, useMatch } from 'react-router-dom';
import styled from 'styled-components';
import { AppHeader } from './Header';
import { useAppWallet } from 'src/states/wallet';
import { message, notification } from 'antd';
import { GlobalVar } from 'src/constants';

export const AppEnterPage: React.FC<{}> = (props) => {
  const match = useMatch('*');
  useAppWallet();
  const [api, contextHolder] = notification.useNotification();
  const [messageHandler, messageCtx] = message.useMessage();
  GlobalVar.notification = api;
  GlobalVar.message = messageHandler;
  return (
    <PageStyle className={`page-${match?.pathname.replace(/\//g, '') || ''}`}>
      <AppHeader />
      <Outlet />
      {contextHolder}
      {messageCtx}
    </PageStyle>
  );
};

const PageStyle = styled.div`
  min-height: 100vh;
  width: 100%;
`;
