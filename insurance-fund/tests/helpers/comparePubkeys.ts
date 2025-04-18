import {PublicKey} from "@solana/web3.js";

function comparePubkeys(a: PublicKey, b: PublicKey): number {
    return a.toBase58().localeCompare(b.toBase58());
}
export default comparePubkeys;