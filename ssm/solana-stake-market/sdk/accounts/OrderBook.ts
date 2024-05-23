/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import * as beetSolana from '@metaplex-foundation/beet-solana'

/**
 * Arguments used to create {@link OrderBook}
 * @category Accounts
 * @category generated
 */
export type OrderBookArgs = {
  tvl: beet.bignum
  bids: beet.bignum
  globalNonce: beet.bignum
  totalTrades: beet.bignum
}

export const orderBookDiscriminator = [55, 230, 125, 218, 149, 39, 65, 248]
/**
 * Holds the data for the {@link OrderBook} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export class OrderBook implements OrderBookArgs {
  private constructor(
    readonly tvl: beet.bignum,
    readonly bids: beet.bignum,
    readonly globalNonce: beet.bignum,
    readonly totalTrades: beet.bignum
  ) {}

  /**
   * Creates a {@link OrderBook} instance from the provided args.
   */
  static fromArgs(args: OrderBookArgs) {
    return new OrderBook(
      args.tvl,
      args.bids,
      args.globalNonce,
      args.totalTrades
    )
  }

  /**
   * Deserializes the {@link OrderBook} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0
  ): [OrderBook, number] {
    return OrderBook.deserialize(accountInfo.data, offset)
  }

  /**
   * Retrieves the account info from the provided address and deserializes
   * the {@link OrderBook} from its data.
   *
   * @throws Error if no account info is found at the address or if deserialization fails
   */
  static async fromAccountAddress(
    connection: web3.Connection,
    address: web3.PublicKey,
    commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig
  ): Promise<OrderBook> {
    const accountInfo = await connection.getAccountInfo(
      address,
      commitmentOrConfig
    )
    if (accountInfo == null) {
      throw new Error(`Unable to find OrderBook account at ${address}`)
    }
    return OrderBook.fromAccountInfo(accountInfo, 0)[0]
  }

  /**
   * Provides a {@link web3.Connection.getProgramAccounts} config builder,
   * to fetch accounts matching filters that can be specified via that builder.
   *
   * @param programId - the program that owns the accounts we are filtering
   */
  static gpaBuilder(
    programId: web3.PublicKey = new web3.PublicKey(
      'sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37'
    )
  ) {
    return beetSolana.GpaBuilder.fromStruct(programId, orderBookBeet)
  }

  /**
   * Deserializes the {@link OrderBook} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [OrderBook, number] {
    return orderBookBeet.deserialize(buf, offset)
  }

  /**
   * Serializes the {@link OrderBook} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return orderBookBeet.serialize({
      accountDiscriminator: orderBookDiscriminator,
      ...this,
    })
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link OrderBook}
   */
  static get byteSize() {
    return orderBookBeet.byteSize
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link OrderBook} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      OrderBook.byteSize,
      commitment
    )
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link OrderBook} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === OrderBook.byteSize
  }

  /**
   * Returns a readable version of {@link OrderBook} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      tvl: (() => {
        const x = <{ toNumber: () => number }>this.tvl
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      bids: (() => {
        const x = <{ toNumber: () => number }>this.bids
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      globalNonce: (() => {
        const x = <{ toNumber: () => number }>this.globalNonce
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
      totalTrades: (() => {
        const x = <{ toNumber: () => number }>this.totalTrades
        if (typeof x.toNumber === 'function') {
          try {
            return x.toNumber()
          } catch (_) {
            return x
          }
        }
        return x
      })(),
    }
  }
}

/**
 * @category Accounts
 * @category generated
 */
export const orderBookBeet = new beet.BeetStruct<
  OrderBook,
  OrderBookArgs & {
    accountDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['tvl', beet.u64],
    ['bids', beet.u64],
    ['globalNonce', beet.u64],
    ['totalTrades', beet.u64],
  ],
  OrderBook.fromArgs,
  'OrderBook'
)
