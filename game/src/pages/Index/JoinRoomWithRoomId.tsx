import { ArrowRightOutlined, CloseCircleFilled } from '@ant-design/icons';
import { Input } from 'antd';
import React, { useEffect, useMemo, useState } from 'react';
import { Outlet, useMatch } from 'react-router-dom';
import styled from 'styled-components';

export const JoinRoomWithRoomId: React.FC<{ className?: string }> = (props) => {
  const [roomId, _roomId] = useState<null | string>(null);

  if (typeof roomId !== 'string') {
    return (
      <PageStyle className={props.className} onClick={() => _roomId('')}>
        JoinRoomWithRoomId
      </PageStyle>
    );
  }
  return (
    <PageStyle className={props.className}>
      <Input value={roomId} onChange={(e) => _roomId(e.target.value)} addonBefore={<CloseCircleFilled onClick={() => _roomId(null)} />} placeholder="#roomId" addonAfter={<ArrowRightOutlined />} />
    </PageStyle>
  );
};

const PageStyle = styled.div`
  display: flex;
  gap: 4px;
  &:hover {
    transform: scale(1) !important;
  }
  .ant-input-outlined {
    background-color: transparent;
  }
`;
