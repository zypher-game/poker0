# poker0
A poker game leveraging RISC0 and PLONK for off-chain proof of game processes, with on-chain validation, built on the Z4 engine.

## Game
- 3 players
- 48 cards ((3~Q) * 4 + A * 3 + 2 * 1)

### Rules
1. Every time, the `Heart 3` is played first
2. Single: the order of size is 2 > A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3
3. Pairs: the order of size is: 2 pair > A pair > K pair > ... > 4 pairs > 3 pairs
4. Connected pairs: three+ connected pairs. For example: 334455, 778899, JJQQKKAA
5. Three: KKK > ... 444 > 333
6. Three with one: 2223 > AAA3 > KKK4 > ... 444K > 333A
7. Three with pair: 22234 > AAA33 > KKKK44 > ... 444KK > 333AA
8. Four with two: KKKK34 > ... 4444KQ > 3333AA
9. Link: 5+ cards. Such as 34567, 3456789
10. Bomb: four cards with same number. AAA is maximum

- Head: The first player to finish playing cards is called the "Head".
- Breakeven: Only 1 card left but no win. At this time, it is not considered a loss and is called "Breakeven".
- Winning or Losing: Every time the "Head" wins, all other players lose except for "Breakeven". The negative player will lose the corresponding number of cards left.
