import { PokerCard } from 'src/pages/Game/states';
import { Poker0Game, ShowCardType } from 'src/types/poker';

const CardTypeConfig = [
  ShowCardType.Single,
  ShowCardType.Pairs,
  ShowCardType.ConnectedPairs,
  ShowCardType.Three,
  ShowCardType.ThreeWithOne,
  ShowCardType.ThreeWithPair,
  ShowCardType.Straight,
  ShowCardType.FourWithTwo,
  ShowCardType.Bomb,
  ShowCardType.AAA,
];

const AAA = 11;
export const RowPoker0Ai = (publicCards: PokerCard[], myCards: PokerCard[], limit: NonNullable<ReturnType<typeof Poker0Game.getCardsTypeAndValue>>): PokerCard[] | null => {
  console.log('RowPoker0Ai', publicCards);
  const myValueData = new Map<number, PokerCard[]>();
  const valueMap = new Map<number, number>();
  const sizeList = new Map<number, PokerCard[][]>();
  myCards.forEach((card) => {
    valueMap.set(card.value, (valueMap.get(card.value) || 0) + 1);
    const list = myValueData.get(card.value) || [];
    list.push(card);
    myValueData.set(card.value, list);
  });
  for (const [value, size] of valueMap) {
    const list = sizeList.get(size) || [];
    // AAA
    if (size === 3 && value === AAA) {
      sizeList.set(AAA, [myValueData.get(value)!]);
    } else {
      list.push(myValueData.get(value)!);
      sizeList.set(size, list);
    }
  }

  const findCard1 = (min: number) => {
    const list = sizeList.get(1) || [];
    for (const cards of list) {
      if (cards[0].value > min) return cards;
    }
  };
  const findCard2 = (min: number) => {
    const list = sizeList.get(2) || [];
    for (const cards of list) {
      if (cards[0].value > min) return cards;
    }
  };
  const findCard3 = (min: number) => {
    const list = sizeList.get(3) || [];
    for (const cards of list) {
      if (cards[0].value > min) return cards;
    }
  };
  const findCard4 = (min: number) => {
    const list = sizeList.get(4) || [];
    for (const cards of list) {
      if (cards[0].value > min) return cards;
    }
  };
  const findCardAAA = () => {
    const list = sizeList.get(AAA) || [];
    if (list.length > 0) return list[0];
  };

  switch (limit.type) {
    case ShowCardType.Pass: {
      return null;
    }
    case ShowCardType.AAA: {
      return null;
    }
    case ShowCardType.Bomb: {
      const c4 = findCard4(limit.value);
      if (c4) return c4;
      const aaa = findCardAAA();
      if (aaa) return aaa;
      return null;
    }
    case ShowCardType.Single: {
      const c1 = findCard1(limit.value);
      if (c1) return c1;

      const c2 = findCard2(limit.value);
      if (c2) return [c2[0]];

      const c3 = findCard3(limit.value);
      if (c3) return [c3[0]];
      break;
    }

    case ShowCardType.Pairs: {
      const c2 = findCard2(limit.value);
      if (c2) return c2;

      const c3 = findCard3(limit.value);
      if (c3) return [c3[0], c3[1]];
      break;
    }

    case ShowCardType.Three: {
      const c3 = findCard3(limit.value);
      if (c3) return c3;
      break;
    }

    case ShowCardType.ConnectedPairs: {
      const len2 = limit.cards.length / 2;
      const c2s = (sizeList.get(2) || []).filter((c) => c[0].value > limit.value);
      const c3s = (sizeList.get(3) || []).filter((c) => c[0].value > limit.value);
      if (c2s.length + c3s.length < len2) break;
      const list = [...c2s];
      c3s.forEach((cs) => list.push([cs[0], cs[1]]));
      list.sort((c1, c2) => c1[0].value - c2[0].value);
      let temp: typeof c2s = [];
      for (const cs of list) {
        const last = temp[temp.length - 1];
        if (last) {
          if (cs[0].value - last[0].value === 1) {
            temp.push(cs);
          } else {
            temp = [];
          }
        } else {
          temp.push(cs);
        }
      }
      if (temp.length >= len2) {
        const revert: PokerCard[] = [];
        temp.forEach((cs, index) => {
          if (index >= len2) return;
          revert.push(...cs);
        });
        return revert;
      }
      break;
    }

    case ShowCardType.FourWithTwo: {
      const c4 = findCard4(limit.value);
      if (!c4) break;
      const c2 = findCard2(-1);
      if (!c2) break;
      return [...c4].concat(c2);
    }

    case ShowCardType.Straight: {
      const c1s = (sizeList.get(2) || []).filter((c) => c[0].value > limit.value);
      const c2s = (sizeList.get(2) || []).filter((c) => c[0].value > limit.value);
      const c3s = (sizeList.get(3) || []).filter((c) => c[0].value > limit.value);
      if (c1s.length + c2s.length + c3s.length < limit.cards.length) break;
      const list = c1s.map((c) => c[0]);
      c2s.forEach((cs) => list.push(cs[0]));
      c3s.forEach((cs) => list.push(cs[0]));
      list.sort((c1, c2) => c1.value - c2.value);
      let temp: PokerCard[] = [];
      for (const cs of list) {
        const last = temp[temp.length - 1];
        if (last) {
          if (cs.value - last.value === 1) {
            temp.push(cs);
          } else {
            temp = [];
          }
        } else {
          temp.push(cs);
        }
      }
      if (temp.length >= limit.cards.length) {
        return temp.slice(0, limit.cards.length);
      }
      break;
    }

    case ShowCardType.ThreeWithOne: {
      const c3 = findCard3(limit.value);
      if (!c3) break;
      const c1 = findCard1(-1);
      if (!c1) break;
      return [...c3].concat(c1);
    }

    case ShowCardType.ThreeWithPair: {
      const c3 = findCard3(limit.value);
      if (!c3) break;
      const c2 = findCard2(-1);
      if (!c2) break;
      return [...c3].concat(c2);
    }
  }
  const c4 = findCard4(-1);
  if (c4) return c4;

  const aaa = findCardAAA();
  if (aaa) return aaa;
  return null;
};
