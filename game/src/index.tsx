import React from 'react';
import ReactDOM from 'react-dom/client';
import './assets/scss/index.scss';
import '@rainbow-me/rainbowkit/styles.css';
import { WagmiProvider } from 'wagmi';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RecoilRoot } from 'recoil';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import { RainbowKitProvider, darkTheme } from '@rainbow-me/rainbowkit';
import { PageIndex } from './pages/Index';
import { AppEnterPage } from './pages/App';
import { GamePage } from './pages/Game';
import { RainbowKitConfig } from './wagmi.config';
import { PokerAvatar } from './components/PokerAvatar';

const queryClient = new QueryClient();
const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);

root.render(
  <React.StrictMode>
    <WagmiProvider config={RainbowKitConfig}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider coolMode avatar={PokerAvatar} theme={darkTheme()}>
          <ConfigProvider
            theme={{
              token: {
                colorPrimary: '#882db4',
                colorBgElevated: '#4157be',
                colorBgContainer: '#002e6b',
                colorText: '#ececec',
                colorTextDescription: '#cbcbcb',
              },
              components: {
                Table: {
                  borderColor: '#ffffff14',
                  headerSplitColor: '#ffffff14',
                  colorText: '#00c192',
                  headerColor: '#00ffc1',
                  cellPaddingBlockSM: 1,
                  cellPaddingInlineSM: 1,
                  rowHoverBg: '#002244',
                },
                Segmented: {
                  trackBg: '#f5f5f517',
                  itemColor: '#ffffff85',
                  itemSelectedBg: '#9dbbff',
                  itemSelectedColor: '#003e54e0',
                },
              },
            }}
          >
            <RecoilRoot>
              <BrowserRouter>
                <Routes>
                  <Route path="/" element={<AppEnterPage />}>
                    <Route index element={<PageIndex />} />
                    <Route path="game/:roomId?" element={<GamePage />} />
                  </Route>
                </Routes>
              </BrowserRouter>
            </RecoilRoot>
          </ConfigProvider>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  </React.StrictMode>,
);
