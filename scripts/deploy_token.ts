import { Address, Contract, Keypair, nativeToScVal } from '@stellar/stellar-sdk';
import { AddressBook } from '../utils/address_book.js';
import { deploySorobanToken } from '../utils/contract.js';
import { Token } from '../utils/tokens_book.js';
import { invoke } from '../utils/tx.js';

/**
 * Deploy a token contract and initialize it
 * @param name Name of the token
 * @param symbol Symbol of the token
 * @param addressBook AddressBook instance
 * @param source Keypair of the source account
 */
export async function deployToken(
  name: string,
  symbol: string,
  icon: string,
  source: Keypair,
  addressBook: AddressBook,
): Promise<Token> {
  const contractId = await deploySorobanToken('token', addressBook, source);

  if (!contractId) {
    throw new Error(`Token ${symbol} deployment failed: no contract id returned`);
  }

  // Initializing Token
  const tokenInitParams = [
    new Address(source.publicKey()).toScVal(),
    nativeToScVal(7, { type: 'u32' }),
    nativeToScVal(name, { type: 'string' }),
    nativeToScVal(symbol, { type: 'string' }),
  ];

  const contractInstance = new Contract(contractId);
  const contractOperation = contractInstance.call('initialize', ...tokenInitParams);
  const result = await invoke(contractOperation, source, false);

  if (result.status !== 'SUCCESS') {
    throw new Error(`Token ${symbol} deployment failed with contractId: ${contractId}`);
  }

  return {
    name: name,
    contract: contractId,
    code: symbol,
    icon: icon,
    decimals: 7,
  };
}
