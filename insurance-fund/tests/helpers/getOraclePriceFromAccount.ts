import getOraclePrice from "./getOraclePrice";

const feeds: Map<string, string> = new Map([
    ["7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE", "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"],
    ["Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX", "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a"],
]);

export default function getOraclePriceFromAccount(account: string) {
    return getOraclePrice(feeds.get(account));
}