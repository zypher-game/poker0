import { UnwrapRecoilValue } from 'recoil';
import { PageGameDataState, PokerCard } from 'src/pages/Game/states';

export enum PokerSuit {
  Unknown, // ........ 0 (Unkonwn)
  Club, // ............ 4 (♣)
  Diamond, // ........ 3 (♦)
  Heart, // .......... 2 (♥)
  Spade, // .......... 1 (♠)
}
export const PokerSuitValue = {
  [PokerSuit.Unknown]: '',
  [PokerSuit.Club]: '♣',
  [PokerSuit.Diamond]: '♦',
  [PokerSuit.Heart]: '♥',
  [PokerSuit.Spade]: '♠',
};

export const PokerValueMap: Record<number, string> = {
  0: '3',
  1: '4',
  2: '5',
  3: '6',
  4: '7',
  5: '8',
  6: '9',
  7: '10',
  8: 'J',
  9: 'Q',
  10: 'K',
  11: 'A',
  20: '2',
};

export const PokerSuitMap = {
  [PokerSuit.Unknown]: '',
  [PokerSuit.Club]: '♣',
  [PokerSuit.Diamond]: '♦',
  [PokerSuit.Heart]: '♥',
  [PokerSuit.Spade]: '♠',
};

export const PokerConstant: Array<{ value: number; suit: number; key: number }> = [];
// export type PokerCard = (typeof PokerConstant)[number];
for (let suit = 1; suit < 5; suit++) {
  for (let value = 0; value < 12; value++) {
    if (value === 11 && suit === 0) continue; // A
    PokerConstant.push({ value, suit, key: suit * 100 + value });
  }
}
PokerConstant.push({ value: 20, suit: 1, key: 1 * 100 + 20 }); // 2

export enum ShowCardType {
  'Pass' = 1000,
  'Single' = 0,
  'Pairs' = 1,
  'ConnectedPairs' = 2,
  'Three' = 3,
  'ThreeWithOne' = 4,
  'ThreeWithPair' = 5,
  'Straight' = 6,
  'FourWithTwo' = 7,
  'Bomb' = 8,
  'AAA' = 9,
}

export class Poker0Game {
  static fmtRound(data: UnwrapRecoilValue<typeof PageGameDataState>) {
    const revert = { round: 0, turn: 0 };
    if (data.consumeLogs.length === 0) return data;
    // 找到轮次(第一个action 或者 两个pass之后的那个action)
    let roundAction = data.consumeLogs[0];
    const logs: typeof data.logs = [];
    const consumeCards: Record<string, PokerCard[]> = {};
    logs.unshift({ name: `${revert.round + 1}`, logs: [] });
    let last = roundAction;
    for (let i = 0; i < data.consumeLogs.length; i++) {
      const act = data.consumeLogs[i];
      consumeCards[act.player] = consumeCards[act.player] || [];
      if (typeof act.round !== 'number') act.round = revert.round;
      if (typeof act.turn !== 'number') act.turn = revert.turn;
      consumeCards[act.player].push(...(act.cards || []));
      logs[0].logs.unshift(act);
      revert.turn++;
      if (last.action === 'pass' && act.action === 'pass') {
        revert.round++;
        revert.turn = 0;
        logs.unshift({ name: `${revert.round + 1}`, logs: [] });
        roundAction = data.consumeLogs[i + 1] || roundAction;
      }
      last = act;
    }
    data.logs = logs;
    data.consumeCards = consumeCards;
    data.consumeRound = revert.round;
    data.consumeTurn = revert.turn;
    return data;
  }
  static getCardsTypeAndValue(cards: PokerCard[]) {
    console.log('getCardsTypeAndValue', cards);
    if (cards.length === 0) return null;
    // 从小到大排序
    cards.sort((c1, c2) => c1.value - c2.value);
    const v0 = cards[0].value;

    const revert = { cards, value: v0, type: ShowCardType.Single };

    // 单牌[1]
    if (cards.length === 1) {
      [revert.value, revert.type] = [v0, ShowCardType.Single];
      return revert;
    }
    const valueMap = new Map<number, number>();
    cards.forEach((card) => {
      valueMap.set(card.value, (valueMap.get(card.value) || 0) + 1);
    });
    const s0 = valueMap.get(v0);
    // 对子: [2]
    if (cards.length === 2) {
      if (valueMap.size === 1) {
        [revert.value, revert.type] = [v0, ShowCardType.Pairs];
        return revert;
      }
      return null;
    }
    // 三带0: [3]
    // 炸弹: [3A]
    if (cards.length === 3) {
      if (valueMap.size === 1) {
        // 三个A,是炸弹[3]
        if (v0 === 11) {
          [revert.value, revert.type] = [v0, ShowCardType.AAA];
          return revert;
        }
        // 三带0[3]
        [revert.value, revert.type] = [v0, ShowCardType.Three];
        return revert;
      }
      return null;
    }
    // 炸弹: [4]
    // 三带一: [1,3] [3,1]
    if (cards.length === 4) {
      // 炸弹[4]
      if (valueMap.size === 1) {
        [revert.value, revert.type] = [v0, ShowCardType.Bomb];
        return revert;
      }
      if (valueMap.size === 2) {
        // 四张里有两个面值, 只会是 [2,2]和[1,3]和[3,1]
        if (s0 === 3) {
          [revert.value, revert.type] = [v0, ShowCardType.ThreeWithOne];
          return revert;
        }
        if (s0 === 1) {
          const newCards = [...cards];
          newCards.push(newCards[0]);
          newCards.shift();
          [revert.cards, revert.value, revert.type] = [newCards, cards[1].value, ShowCardType.ThreeWithOne];
          return revert;
        }
        return null; // 不能[2,2]
      }
      return null;
    }
    // 顺子
    if (cards.length >= 5 && valueMap.size === cards.length) {
      let isStraight = true;
      for (let i = 1; i < cards.length; i++) {
        if (cards[i].value - cards[i - 1].value === 1) continue;
        isStraight = false;
      }
      if (isStraight) {
        [revert.value, revert.type] = [v0, ShowCardType.Straight];
        return revert;
      }
      return null;
    }
    // 三带一对: [2,3] [3,2]
    // =======不允许: 三带二. 四带一
    if (cards.length === 5) {
      // [1,4] [2,3] [3,2] [4,1]
      if (valueMap.size === 2) {
        // [2, 3] 三带一对
        if (s0 === 2) {
          const newCards = [...cards];
          newCards.push(newCards[0], newCards[1]);
          newCards.splice(0, 2);
          [revert.cards, revert.value, revert.type] = [newCards, cards[4].value, ShowCardType.ThreeWithPair];
          return revert;
        }
        // [3, 2] 三带一对
        if (s0 === 3) {
          [revert.value, revert.type] = [cards[4].value, ShowCardType.ThreeWithPair];
          return revert;
        }
        return null;
      }
      return null;
    }
    // 四带二: [2,4] [4,2] [1,1,4] [1,4,1] [4,1,1]
    // 三连对: [2,2,2]
    if (cards.length === 6) {
      if (valueMap.size === 2) {
        // [2,4]
        if (s0 === 2) {
          const newCards = [...cards];
          newCards.push(newCards[0], newCards[1]);
          newCards.splice(0, 2);
          [revert.cards, revert.value, revert.type] = [newCards, cards[5].value, ShowCardType.FourWithTwo];
          return revert;
        }
        // [4,2]
        if (s0 === 4) {
          [revert.value, revert.type] = [v0, ShowCardType.FourWithTwo];
          return revert;
        }
        return null;
      }
      if (valueMap.size === 3) {
        // [4,1,1] 四带二
        if (s0 === 4) {
          [revert.value, revert.type] = [v0, ShowCardType.FourWithTwo];
          return revert;
        }
        // [1,4,1]
        if (valueMap.get(cards[1].value) === 4) {
          const newCards = [...cards];
          newCards.push(newCards[0]);
          newCards.splice(0, 1);
          [revert.cards, revert.value, revert.type] = [newCards, cards[1].value, ShowCardType.FourWithTwo];
          return revert;
        }
        // [1,1,4]
        if (valueMap.get(cards[5].value) === 4) {
          const newCards = [...cards];
          newCards.push(newCards[0], newCards[1]);
          newCards.splice(0, 2);
          [revert.cards, revert.value, revert.type] = [newCards, cards[5].value, ShowCardType.FourWithTwo];
          return revert;
        }
        // [2,2,2]
        if (s0 === 2 && valueMap.get(cards[2].value) === 2 && cards[2].value - v0 === 1 && cards[5].value - v0 === 2) {
          [revert.value, revert.type] = [v0, ShowCardType.ConnectedPairs];
          return revert;
        }
        return null;
      }
      return null;
    }
    if (cards.length >= 8 && cards.length % 2 === 0) {
      const halfLen = cards.length / 2;
      if (valueMap.size !== halfLen) return null;
      let isStraight = true;
      for (let i = 2; i < halfLen; i++) {
        if (cards[i].value - cards[i - 2].value === 1) continue;
        isStraight = false;
      }
      if (isStraight) {
        [revert.value, revert.type] = [v0, ShowCardType.ConnectedPairs];
        return revert;
      }
      return null;
    }
    return null;
  }

  static shuffle() {
    const originCards = [...PokerConstant];
    const cards: PokerCard[] = [];
    while (originCards.length > 0) {
      const index = Math.floor(Math.random() * originCards.length);
      const card = originCards.splice(index, 1)[0];
      cards.push(card);
    }
    return cards;
  }

  static isEnd(pokers: PokerCard[][]) {
    let hasZeroLen = false;
    let hasLen = false;
    pokers.forEach((cs) => {
      if (cs.length === 0) {
        hasZeroLen = true;
      } else {
        hasLen = true;
      }
    });
    return hasLen && hasZeroLen;
  }
}
