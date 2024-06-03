import { Collapse, CollapseProps } from 'antd';
import React, { useEffect, useMemo, useState } from 'react';
import { Outlet, useMatch } from 'react-router-dom';
import styled from 'styled-components';

const rules = [
  { label: 'Heart 3', children: `Every time, the Heart 3 is played first` },
  { label: 'Single', children: `the order of size is 2 > A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3` },
  { label: 'Pairs', children: `the order of size is: A pair > K pair > ... > 4 pairs > 3 pairs` },
  { label: 'Three', children: `AAA > KKK > ... 444 > 333` },
  { label: 'Connected pairs', children: `three+ connected pairs. For example: 223344, 778899, JJQQKKAA` },
  { label: 'Three with one', children: `AAA3 > KKK4 > ... 444K > 333A` },
  { label: 'Three with pair', children: `AAA33 > KKKK44 > ... 444KK > 333AA` },
  { label: 'Four with two', children: `KKKK34 > QQQQ45 > ... 4444KQ > 3333AA` },
  { label: 'Straight', children: `5+ cards. Such as 34567, 3456789` },
  { label: 'Bomb', children: `four cards with same number. AAA is maximum` },
];

export const PokerRulesTip: React.FC<{}> = (props) => {
  return (
    <PageStyle>
      {rules.map((rule) => {
        return (
          <div key={rule.label}>
            <span>{rule.label}:</span>
            {rule.children}
          </div>
        );
      })}
    </PageStyle>
  );
};

const PageStyle = styled.div`
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  color: #bb76d0;
  > div {
    > span {
      font-weight: bold;
      color: #6e208c;
      margin-right: 4px;
    }
  }
`;
