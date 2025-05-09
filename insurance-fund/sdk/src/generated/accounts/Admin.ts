/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as web3 from '@solana/web3.js'
import * as beetSolana from '@metaplex-foundation/beet-solana'
import * as beet from '@metaplex-foundation/beet'
import { Permissions, permissionsBeet } from '../types/Permissions'

/**
 * Arguments used to create {@link Admin}
 * @category Accounts
 * @category generated
 */
export type AdminArgs = {
  address: web3.PublicKey
  permissions: Permissions
}

export const adminDiscriminator = [244, 158, 220, 65, 8, 73, 4, 65]
/**
 * Holds the data for the {@link Admin} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export class Admin implements AdminArgs {
  private constructor(
    readonly address: web3.PublicKey,
    readonly permissions: Permissions
  ) {}

  /**
   * Creates a {@link Admin} instance from the provided args.
   */
  static fromArgs(args: AdminArgs) {
    return new Admin(args.address, args.permissions)
  }

  /**
   * Deserializes the {@link Admin} from the data of the provided {@link web3.AccountInfo}.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static fromAccountInfo(
    accountInfo: web3.AccountInfo<Buffer>,
    offset = 0
  ): [Admin, number] {
    return Admin.deserialize(accountInfo.data, offset)
  }

  /**
   * Retrieves the account info from the provided address and deserializes
   * the {@link Admin} from its data.
   *
   * @throws Error if no account info is found at the address or if deserialization fails
   */
  static async fromAccountAddress(
    connection: web3.Connection,
    address: web3.PublicKey,
    commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig
  ): Promise<Admin> {
    const accountInfo = await connection.getAccountInfo(
      address,
      commitmentOrConfig
    )
    if (accountInfo == null) {
      throw new Error(`Unable to find Admin account at ${address}`)
    }
    return Admin.fromAccountInfo(accountInfo, 0)[0]
  }

  /**
   * Provides a {@link web3.Connection.getProgramAccounts} config builder,
   * to fetch accounts matching filters that can be specified via that builder.
   *
   * @param programId - the program that owns the accounts we are filtering
   */
  static gpaBuilder(
    programId: web3.PublicKey = new web3.PublicKey(
      'rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D'
    )
  ) {
    return beetSolana.GpaBuilder.fromStruct(programId, adminBeet)
  }

  /**
   * Deserializes the {@link Admin} from the provided data Buffer.
   * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
   */
  static deserialize(buf: Buffer, offset = 0): [Admin, number] {
    return adminBeet.deserialize(buf, offset)
  }

  /**
   * Serializes the {@link Admin} into a Buffer.
   * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
   */
  serialize(): [Buffer, number] {
    return adminBeet.serialize({
      accountDiscriminator: adminDiscriminator,
      ...this,
    })
  }

  /**
   * Returns the byteSize of a {@link Buffer} holding the serialized data of
   * {@link Admin}
   */
  static get byteSize() {
    return adminBeet.byteSize
  }

  /**
   * Fetches the minimum balance needed to exempt an account holding
   * {@link Admin} data from rent
   *
   * @param connection used to retrieve the rent exemption information
   */
  static async getMinimumBalanceForRentExemption(
    connection: web3.Connection,
    commitment?: web3.Commitment
  ): Promise<number> {
    return connection.getMinimumBalanceForRentExemption(
      Admin.byteSize,
      commitment
    )
  }

  /**
   * Determines if the provided {@link Buffer} has the correct byte size to
   * hold {@link Admin} data.
   */
  static hasCorrectByteSize(buf: Buffer, offset = 0) {
    return buf.byteLength - offset === Admin.byteSize
  }

  /**
   * Returns a readable version of {@link Admin} properties
   * and can be used to convert to JSON and/or logging
   */
  pretty() {
    return {
      address: this.address.toBase58(),
      permissions: 'Permissions.' + Permissions[this.permissions],
    }
  }
}

/**
 * @category Accounts
 * @category generated
 */
export const adminBeet = new beet.BeetStruct<
  Admin,
  AdminArgs & {
    accountDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['address', beetSolana.publicKey],
    ['permissions', permissionsBeet],
  ],
  Admin.fromArgs,
  'Admin'
)
