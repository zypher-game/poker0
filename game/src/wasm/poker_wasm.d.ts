/* tslint:disable */
/* eslint-disable */
/**
* generate keypair
* @returns {any}
*/
export function generate_key(): any;
/**
* compute masked to revealed card and the revealed proof
* @param {string} sk
* @param {any} card
* @returns {any}
*/
export function reveal_card(sk: string, card: any): any;
/**
* compute masked to revealed card and the revealed proof
* @param {string} sk
* @param {any} card
* @returns {any}
*/
export function batch_reveal_card(sk: string, card: any): any;
/**
* unmask the card use all reveals
* @param {any} card
* @param {any} reveals
* @returns {number}
*/
export function unmask_card(card: any, reveals: any): number;
/**
* batch unmask the card use all reveals
* @param {any} card
* @param {any} reveals
* @returns {any}
*/
export function batch_unmask_card(card: any, reveals: any): any;
/**
* @param {any} player_env
* @returns {string}
*/
export function create_play_env(player_env: any): string;
